/// 匹配模块：扫描区块并查找匹配图案的起始位置
/// 
/// 采用即时计算策略，不预先缓存所有区块结果，而是按需判断每个位置，
/// 遇到不匹配立即跳过，从而大幅降低内存使用，适合大规模扫描。
use crate::slime_lib::is_slime_chunk;
use crate::preprocess::Preprocessed;

/// 在指定范围内查找匹配图案的区块位置
/// - pre: 预处理结果（包含扫描范围和图案）
/// - 返回: (匹配位置列表, 扫描区块总数)
pub fn find_matches(pre: &Preprocessed) -> (Vec<(i32, i32)>, usize) {
    let seed = pre.seed;
    let limit = pre.limit;

    // 计算总扫描区块数（扩展后范围）
    let x0 = pre.cache_x0;
    let x1 = pre.cache_x1;
    let z0 = pre.cache_z0;
    let z1 = pre.cache_z1;
    let total_blocks = ((x1 - x0 + 1) * (z1 - z0 + 1)) as usize;

    let mut results = Vec::new();

    // 遍历所有可能的起始位置，逐块判断是否匹配图案
    'outer: for start_x in pre.x_start..=pre.x_end {
        for start_z in pre.z_start..=pre.z_end {
            let mut match_ok = true;
            // 检查图案每个位置是否匹配
            'pat: for (r, row) in pre.pattern.iter().enumerate() {
                let z = start_z + r as i32;
                for (c, &expected) in row.iter().enumerate() {
                    if expected == 2 { continue; } // 2 表示忽略该位置
                    let x = start_x + c as i32;
                    // 即时计算该区块是否为史莱姆区块
                    let actual = is_slime_chunk(seed, x, z);
                    let expected_bool = expected == 1;
                    if actual != expected_bool {
                        match_ok = false;
                        break 'pat; // 不匹配，跳过当前起始位置
                    }
                }
            }
            // 图案完全匹配才加入结果
            if match_ok {
                results.push((start_x, start_z));
                if limit > 0 && results.len() >= limit {
                    break 'outer;
                }
            }
        }
    }

    (results, total_blocks)
}
