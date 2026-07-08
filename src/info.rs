use crate::config::{Config, Mode, CountShape};
use crate::preprocess::PreprocessedData;

pub fn build_info(config: &Config, prep: &PreprocessedData) -> String {
    let mut info = String::with_capacity(1024);
    
    info.push_str("==== [Info] ====\n");
    info.push_str(&format!("种子: {}\n", config.world_seed));
    info.push_str(&format!("中心区块: ({}, {})\n", prep.center_block_x, prep.center_block_z));
    info.push_str(&format!("中心世界坐标: x:[{},{}) z:[{},{})\n", 
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
                info.push_str(&format!("匹配目标: {}\n", config.match_target));
            }
        }
        Mode::Count => {
            info.push_str("模式: Count\n");
            info.push_str(&format!("计数形状: {}\n", match config.count_shape {
                CountShape::Square => "正方形",
                CountShape::Circle => "圆形",
            }));
            info.push_str(&format!("区域尺寸: {}\n", config.count_size));
            info.push_str(&format!("目标数量: {}\n", config.count_target));
        }
    }
    
    let total_memory_gib = (prep.total_blocks as f64) / (1024.0 * 1024.0 * 1024.0);
    info.push_str(&format!("预估内存: {:.4} GiB\n", total_memory_gib));
    
    if config.memory_limit_gib > 0.0 {
        info.push_str(&format!("内存限制: {} GiB\n", config.memory_limit_gib));
    } else {
        info.push_str("内存限制: 无限制\n");
    }
    
    if prep.chunk_count > 1 {
        info.push_str("分块处理: 是\n");
        info.push_str(&format!("分块数量: {}\n", prep.chunk_count));
        info.push_str(&format!("每块行数: {}\n", prep.chunk_size / prep.x_count));
        let chunk_memory_gib = (prep.chunk_size as f64) / (1024.0 * 1024.0 * 1024.0);
        info.push_str(&format!("分块内存: {:.4} GiB\n", chunk_memory_gib));
    } else {
        info.push_str("分块处理: 否\n");
    }
    
    match config.mode {
        Mode::Check => {
            info.push_str(&format!("输出网格文件: {}\n", config.output_map));
        }
        Mode::Match => {
            info.push_str(&format!("输出匹配文件: {}\n", config.output_match));
            info.push_str(&format!("输出网格文件: {}\n", config.output_map));
        }
        Mode::Count => {
            info.push_str(&format!("输出计数文件: {}\n", config.output_count));
            info.push_str(&format!("输出网格文件: {}\n", config.output_map));
        }
    }
    
    info.push_str(&format!("输出日志文件: {}\n", config.output_log));
    info.push_str(&format!("终端模式: {:?}\n", config.terminal_mode));
    info.push_str("================\n");
    
    info
}
