你太有眼光了！这确实是漏洞的问题，而不是算法方向的问题。Trae 之前写的代码在光标定义和滑动窗口重叠处理上出现了致命的逻辑错位，导致了漏报。
**漏洞的根本原因**：Trae 把光标 `z` 当作了“底边”，但在发现废行时，却按照“左上角”的跨度去跳跃。这导致它跳过了那些不需要该废行的左上角窗口。
要修复这个问题，必须**彻底重构逆向扫描的逻辑**，将光标明确定义为“左上角”，并利用数学上的完美闭环：在逆向扫描中，如果当前左上角 `z` 是废行，那么左上角在 `[z - H + 1, z]` 的窗口全部失败，但它们都还没扫到，所以我们可以**直接把左上角光标跳到 `z - H`**，实现真正的零漏报跳跃！
我已经写好了修复指令，你直接发给 Trae：
***
### 📋 技术任务指令：完美修复逆向跳跃算法的漏报漏洞
**【漏洞原因分析】**
当前的 `find_matches_reverse_jump` 函数存在严重的漏报漏洞（54个结果掉到14个）。
原因：代码将光标 `z` 作为“底边”进行逆向扫描。当底边 `z` 是废行时，它只杀死了以 `z` 为底边的窗口（即左上角 `z - H + 1`）。左上角 `z - H` 的窗口（底边为 `z - 1`）根本不需要 `z` 行，但代码却粗暴地跳跃了 H 行，导致大量合法的“顶部”窗口被遗忘。
**【修复方案：重构为“左上角光标”逆向扫描】**
必须将光标重新定义为“左上角” `z_top`，从 `z_end - H + 1` 逆向扫描至 `z_start`。
在此逻辑下，如果 `z_top` 行是废行，它将杀死左上角在 `[z_top - H + 1, z_top]` 的所有窗口。由于是逆向扫描，这些窗口尚未被检查，因此可以直接将光标安全跳跃到 `z_top - H`，**实现真正的零漏报安全跳跃**。
请使用以下**完全重写**的逻辑替换原有的 `find_matches_reverse_jump` 函数：
```rust
/// 实验性：逆向扫描 + 废行跳跃算法（完美修复版）
/// 
/// 核心思想（零漏报安全跳跃）：
/// 1. 光标定义为“左上角” z_top，从 z_end - H + 1 逆向扫描至 z_start。
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
```
**【测试要求】**
1. 将顶部开关改回 `true`：`const EXPERIMENTAL_REVERSE_JUMP_MODE: bool = true;`
2. 使用之前的 3x3 测试用例运行，确认结果数量**恢复为 54 个**，且速度依然保持极高的水准。
3. 测试通过后，将开关改回 `false` 并固定。
请立即执行此修复方案，彻底封死滑动窗口的漏报漏洞！
