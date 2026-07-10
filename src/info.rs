use crate::config::{Config, Mode};
use crate::preprocess::PreprocessedData;

pub fn build_info(config: &Config, prep: &PreprocessedData) -> String {
    let mut info = String::with_capacity(1024);
    
    info.push_str("==== [Info] ====\n");
    info.push_str(&format!("种子: {}\n", config.world_seed));
    info.push_str(&format!("中心区块: 区块坐标: ({}, {}) - 世界坐标: x:[{},{}) z:[{},{})\n", 
        prep.center_block_x, prep.center_block_z,
        prep.center_world_x, prep.center_world_x + 16,
        prep.center_world_z, prep.center_world_z + 16));
    info.push_str(&format!("扫描半径: {}\n", config.radius));
    info.push_str(&format!("扫描范围 - 区块: 左上角({}, {}) 右下角({}, {})\n", 
        prep.top_left_block.0, prep.top_left_block.1,
        prep.bottom_right_block.0, prep.bottom_right_block.1));
    info.push_str(&format!("扫描范围 - 世界坐标: 左上角x:[{},{}) z:[{},{}) 右下角x:[{},{}) z:[{},{})\n", 
        prep.top_left_world.0, prep.top_left_world.0 + 16,
        prep.top_left_world.1, prep.top_left_world.1 + 16,
        prep.bottom_right_world.0, prep.bottom_right_world.0 + 16,
        prep.bottom_right_world.1, prep.bottom_right_world.1 + 16));
    info.push_str(&format!("区块总数: {}×{}={}\n", prep.x_count, prep.z_count, prep.total_blocks));
    
    match config.mode {
        Mode::Check => {
            info.push_str("模式: Check\n");
        }
        Mode::Match => {
            info.push_str("模式: Match\n");
            if let Some(p) = &prep.pattern {
                info.push_str(&format!("匹配模式: {}×{}\n", p.width, p.height));
                info.push_str("匹配图案:\n");
                for row in 0..p.height {
                    info.push_str("  ");
                    for col in 0..p.width {
                        if col > 0 {
                            info.push_str(", ");
                        }
                        info.push_str(&p.data[row * p.width + col].to_string());
                    }
                    info.push_str("\n");
                }
                info.push_str(&format!("匹配目标: {}\n", config.match_target));
            }
        }
    }
    
    if config.memory_limit_gib > 0.0 {
        info.push_str(&format!("内存限制: {} GiB\n", config.memory_limit_gib));
    } else {
        info.push_str("内存限制: 无限制\n");
    }
    
    info.push_str(&format!("输出匹配文件: {}\n", config.output_match));
    info.push_str(&format!("输出网格文件: {}\n", config.output_map));
    info.push_str(&format!("输出日志文件: {}\n", config.output_log));
    info.push_str(&format!("终端模式: {:?}\n", config.terminal_mode));
    
    info
}
