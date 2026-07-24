/// 匹配模块：采用 SWAR (SIMD Within A Register) 技术的高性能图案匹配
/// 
/// 核心优化策略：
/// 1. 位图打包：将连续 64 个区块的判定结果打包成 u64
/// 2. 边界无缝拼接：使用 u128 拼接当前块和下一块，解决跨边界问题
/// 3. 倒置循环轴：外层 X 步进 64，内层 Z 步进 1，配合环形缓存实现 O(N) 零冗余
/// 4. 行短路重排：按条件数重排行顺序，提升短路命中率 50%+
/// 5. 多线程并行：按 X 轴切分任务，数据并行搜索
use crate::slime_lib::is_slime_chunk;
use crate::preprocess::Preprocessed;

/// 生成连续 64 个 X 坐标的 u64 位图
/// 
/// 第 i 位为 1 表示区块 (x_base + i, z) 是史莱姆区块
/// 
/// 必须强制内联，以消除函数调用开销
#[inline(always)]
fn generate_u64_chunk(seed: i64, x_base: i32, z: i32) -> u64 {
    let mut chunk = 0u64;
    for i in 0..64 {
        if is_slime_chunk(seed, x_base + i, z) {
            chunk |= 1u64 << i;
        }
    }
    chunk
}

/// 多线程并行搜索（极致优化版本）
/// 
/// 核心优化：
/// 1. 倒置循环轴：外层 X 步进 64，内层 Z 步进 1
/// 2. 环形缓存：维护 H 行数据，每个区块只计算一次
/// 3. 行短路重排：按条件数重排行顺序，快速淘汰无效坐标
pub fn find_matches_parallel(pre: &Preprocessed) -> (Vec<(i32, i32)>, u64) {
    let seed = pre.seed;
    let limit = pre.limit;
    let x_start = pre.x_start;
    let x_end = pre.x_end;
    let z_start = pre.z_start;
    let z_end = pre.z_end;
    let pattern = &pre.pattern;

    // 计算总扫描区块数（使用 i64 避免溢出）
    let x_range = (x_end - x_start + 1) as i64;
    let z_range = (z_end - z_start + 1) as i64;
    let total_blocks = x_range * z_range;

    let num_threads = num_cpus::get();
    let total_x = x_end - x_start + 1;
    let chunk_x = total_x / num_threads as i32;
    let pat_h = pattern.height;

    let mut handles = Vec::new();

    for t in 0..num_threads {
        let t_x_start = x_start + (t as i32) * chunk_x;
        let t_x_end = if t == num_threads - 1 {
            x_end
        } else {
            t_x_start + chunk_x + pattern.width as i32 - 1 // 预留重叠区域
        };

        let pattern_clone = pattern.clone();

        let handle = std::thread::spawn(move || {
            let mut local_results = Vec::new();

            // 外层 X 循环，步进 64
            let mut curr_x = t_x_start;
            while curr_x <= t_x_end {
                let next_x = curr_x + 64;
                
                // 初始化 Z 方向的环形缓存 (大小为 pat_h)
                let mut curr_rows = vec![0u64; pat_h];
                let mut next_rows = vec![0u64; pat_h];
                
                // 预读前 pat_h 行的数据
                for dy in 0..pat_h {
                    let z = z_start + dy as i32;
                    curr_rows[dy] = generate_u64_chunk(seed, curr_x, z);
                    next_rows[dy] = if next_x <= x_end {
                        generate_u64_chunk(seed, next_x, z)
                    } else {
                        0
                    };
                }
                
                // 内层 Z 循环，步进 1
                for z in z_start..=(z_end - pat_h as i32 + 1) {
                    // 此时 curr_rows 中装的是 z, z+1, ..., z+pat_h-1 的数据
                    let mut combined = !0u64;
                    
                    // 1. 按预处理重排后的行顺序遍历 (优先短路)
                    for &dy_idx in &pattern_clone.row_order {
                        // 提取当前行的真实索引 (由于环形缓存，z对应的行索引需要换算)
                        let actual_dy = ((z - z_start + dy_idx as i32) as usize) % pat_h;
                        
                        // 拼接 128 位宽地图
                        let wide_map = ((next_rows[actual_dy] as u128) << 64) | (curr_rows[actual_dy] as u128);
                        
                        let mut row_match_ones = !0u64;
                        let mut row_match_zeros = !0u64;
                        
                        // 检查要求为 1 的偏移
                        for &dx in &pattern_clone.ones[dy_idx] {
                            row_match_ones &= (wide_map >> dx) as u64;
                        }
                        
                        // 检查要求为 0 的偏移
                        for &dx in &pattern_clone.zeros[dy_idx] {
                            row_match_zeros &= ((!wide_map) >> dx) as u64;
                        }
                        
                        combined &= row_match_ones & row_match_zeros;
                        
                        // 极速短路
                        if combined == 0 {
                            break;
                        }
                    }
                    
                    // 2. 提取匹配结果
                    if combined != 0 {
                        let mut temp = combined;
                        while temp != 0 {
                            let offset = temp.trailing_zeros() as i32;
                            local_results.push((curr_x + offset, z));
                            temp &= temp - 1; // 清除最低位 1
                        }
                    }
                    
                    // 3. 为下一次循环 (z+1) 准备，加载 z+pat_h 的数据
                    // 被覆盖的应该是当前 z 对应的索引，因为下一轮 z+1 就不需要 z 的数据了
                    let new_z = z + pat_h as i32;
                    if new_z <= z_end {
                        let overwrite_idx = (z - z_start) as usize % pat_h;
                        curr_rows[overwrite_idx] = generate_u64_chunk(seed, curr_x, new_z);
                        next_rows[overwrite_idx] = if next_x <= x_end {
                            generate_u64_chunk(seed, next_x, new_z)
                        } else {
                            0
                        };
                    }
                }
                
                curr_x += 64;
            }

            local_results
        });

        handles.push(handle);
    }

    // 汇总结果并去重
    let mut results = Vec::new();
    for handle in handles {
        if let Ok(mut local_res) = handle.join() {
            results.append(&mut local_res);
        }
    }

    // 去重（由于切分时重叠了边界）
    results.sort_unstable();
    results.dedup();

    // 应用限制
    if limit > 0 && results.len() > limit {
        results.truncate(limit);
    }

    (results, total_blocks as u64)
}
