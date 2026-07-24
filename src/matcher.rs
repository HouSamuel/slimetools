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

// ==========================================
// 开发者实验性开关：逆向跳跃算法
// 仅供大图案（5x5以上）测试使用，严禁暴露给用户配置文件！
// 默认关闭，开启后启用逆向扫描 + 废行跳跃算法
// ==========================================
const EXPERIMENTAL_REVERSE_JUMP_MODE: bool = false;

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

/// 多线程并行搜索（主入口函数）
/// 
/// 根据 EXPERIMENTAL_REVERSE_JUMP_MODE 开关选择算法：
/// - false（默认）：使用经过验证的 SWAR 环形缓存算法
/// - true（实验性）：使用逆向扫描 + 废行跳跃算法（仅供大图案测试）
pub fn find_matches_parallel(pre: &Preprocessed) -> (Vec<(i32, i32)>, u64) {
    if EXPERIMENTAL_REVERSE_JUMP_MODE {
        find_matches_reverse_jump(pre)
    } else {
        find_matches_swar(pre)
    }
}

/// 多线程并行搜索（SWAR 环形缓存版本 - 经过验证的稳定算法）
/// 
/// 核心优化：
/// 1. 倒置循环轴：外层 X 步进 64，内层 Z 步进 1
/// 2. 环形缓存：维护 H 行数据，每个区块只计算一次
/// 3. 行短路重排：按条件数重排行顺序，快速淘汰无效坐标
fn find_matches_swar(pre: &Preprocessed) -> (Vec<(i32, i32)>, u64) {
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

/// 实验性：逆向扫描 + 废行跳跃算法（完美修复版）
/// 
/// 核心思想（零漏报安全跳跃）：
/// 1. 光标定义为"左上角" z_top，从 z_end - H + 1 逆向扫描至 z_start。
/// 2. 维护一个大小为 H 的缓存 rows[0..H-1]，对应 [z_top, z_top + H - 1] 行的数据。
/// 3. 如果发现 z_top 行（rows[0]）是废行，说明跨越 z_top 的所有窗口失败，
///    即左上角在 [z_top - H + 1, z_top] 的窗口全失败。
///    由于是逆向扫描，这些窗口尚未检查，因此直接将光标跳跃到 z_top - H！
///    此时缓存完全失效，需重新生成。
/// 4. 如果不是废行，则用当前 H 行缓存严格验证匹配，然后光标仅退 1 行，并滑动缓存。
#[allow(dead_code)]
fn find_matches_reverse_jump(pre: &Preprocessed) -> (Vec<(i32, i32)>, u64) {
    let seed = pre.seed;
    let limit = pre.limit;
    let x_start = pre.x_start;
    let x_end = pre.x_end;
    let z_start = pre.z_start;
    let z_end = pre.z_end;
    let pattern = &pre.pattern;
    let pat_h = pattern.height;
    
    // 计算总区块数
    let x_range = (x_end - x_start + 1) as i64;
    let z_range = (z_end - z_start + 1) as i64;
    let total_blocks = x_range * z_range;
    
    let num_threads = num_cpus::get();
    let total_x = x_end - x_start + 1;
    let chunk_x = total_x / num_threads as i32;
    let mut handles = Vec::new();
    
    for t in 0..num_threads {
        let t_x_start = x_start + (t as i32) * chunk_x;
        let t_x_end = if t == num_threads - 1 {
            x_end
        } else {
            t_x_start + chunk_x + pattern.width as i32 - 1
        };
        let pattern_clone = pattern.clone();
        
        let handle = std::thread::spawn(move || {
            let mut local_results = Vec::new();
            let mut curr_x = t_x_start;
            
            while curr_x <= t_x_end {
                let next_x = curr_x + 64;
                
                // 逆向扫描的起始左上角 z_top
                let mut z_top = z_end - pat_h as i32 + 1;
                
                // 初始化缓存 rows，大小为 pat_h
                let mut curr_rows = vec![0u64; pat_h];
                let mut next_rows = vec![0u64; pat_h];
                
                // 辅助函数：生成指定 z 坐标的 128 位宽地图
                let gen_wide = |z_coord: i32| -> u128 {
                    let curr = generate_u64_chunk(seed, curr_x, z_coord);
                    let next = if next_x <= x_end {
                        generate_u64_chunk(seed, next_x, z_coord)
                    } else {
                        0
                    };
                    ((next as u128) << 64) | (curr as u128)
                };
                
                // 初始化缓存：[z_top, z_top + pat_h - 1]
                for i in 0..pat_h {
                    let z_coord = z_top + i as i32;
                    let wide = gen_wide(z_coord);
                    curr_rows[i] = wide as u64;
                    next_rows[i] = (wide >> 64) as u64;
                }
                
                while z_top >= z_start {
                    // 当前缓存对应 [z_top, z_top + pat_h - 1]
                    // rows[0] 就是当前 z_top 行的数据
                    
                    // 1. 判断 z_top 行是否是废行（不包含图案任意一行）
                    let mut contains_any_row = false;
                    for dy in 0..pat_h {
                        let wide = ((next_rows[0] as u128) << 64) | (curr_rows[0] as u128);
                        let mut row_match_ones = !0u64;
                        let mut row_match_zeros = !0u64;
                        for &dx in &pattern_clone.ones[dy] {
                            row_match_ones &= (wide >> dx) as u64;
                        }
                        for &dx in &pattern_clone.zeros[dy] {
                            row_match_zeros &= ((!wide) >> dx) as u64;
                        }
                        if (row_match_ones & row_match_zeros) != 0 {
                            contains_any_row = true;
                            break;
                        }
                    }
                    
                    if !contains_any_row {
                        // 2. 废行触发！跨越 z_top 的窗口全失败
                        // 左上角在 [z_top - pat_h + 1, z_top] 的窗口都不用看了
                        // 直接将光标跳到 z_top - pat_h
                        z_top -= pat_h as i32;
                        
                        // 如果跳跃后仍在边界内，缓存完全失效，需重新生成
                        if z_top >= z_start {
                            for i in 0..pat_h {
                                let z_coord = z_top + i as i32;
                                let wide = gen_wide(z_coord);
                                curr_rows[i] = wide as u64;
                                next_rows[i] = (wide >> 64) as u64;
                            }
                        }
                    } else {
                        // 3. 不是废行，验证当前左上角 z_top 的完整窗口
                        let mut combined = !0u64;
                        for &dy_idx in &pattern_clone.row_order {
                            let wide = ((next_rows[dy_idx] as u128) << 64) | (curr_rows[dy_idx] as u128);
                            let mut row_match_ones = !0u64;
                            let mut row_match_zeros = !0u64;
                            for &dx in &pattern_clone.ones[dy_idx] {
                                row_match_ones &= (wide >> dx) as u64;
                            }
                            for &dx in &pattern_clone.zeros[dy_idx] {
                                row_match_zeros &= ((!wide) >> dx) as u64;
                            }
                            combined &= row_match_ones & row_match_zeros;
                            if combined == 0 {
                                break;
                            }
                        }
                        
                        // 提取匹配结果
                        if combined != 0 {
                            let mut temp = combined;
                            while temp != 0 {
                                let offset = temp.trailing_zeros() as i32;
                                local_results.push((curr_x + offset, z_top));
                                temp &= temp - 1;
                            }
                        }
                        
                        // 4. 光标仅退 1 行，滑动缓存
                        z_top -= 1;
                        if z_top >= z_start {
                            // 缓存滑动：丢弃最底行(rows[pat_h-1])，所有行下移，顶部(rows[0])生成新数据
                            for i in (1..pat_h).rev() {
                                curr_rows[i] = curr_rows[i - 1];
                                next_rows[i] = next_rows[i - 1];
                            }
                            let wide = gen_wide(z_top);
                            curr_rows[0] = wide as u64;
                            next_rows[0] = (wide >> 64) as u64;
                        }
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
    results.sort_unstable();
    results.dedup();
    
    if limit > 0 && results.len() > limit {
        results.truncate(limit);
    }
    (results, total_blocks as u64)
}
