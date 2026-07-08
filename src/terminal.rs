use crate::config::{Config, Mode, CountShape, TerminalMode};
use crate::preprocess::PreprocessedData;
use crate::matcher::MatchResult;
use crate::counter::CountResult;
use std::time::Duration;

fn is_enabled(config: &Config) -> bool {
    config.terminal_mode != TerminalMode::None
}

fn is_full(config: &Config) -> bool {
    config.terminal_mode == TerminalMode::Full
}

pub fn print_info(config: &Config, prep: &PreprocessedData) {
    if !is_enabled(config) {
        return;
    }
    
    println!("==== [Info] ====");
    println!("种子: {}", config.world_seed);
    println!("中心区块: ({}, {})", prep.center_block_x, prep.center_block_z);
    println!("中心世界坐标: x:[{},{}) z:[{},{})", 
        prep.center_world_x, prep.center_world_x + 16,
        prep.center_world_z, prep.center_world_z + 16);
    println!("扫描半径: {}", config.radius);
    println!("扫描范围 - 区块: 左上角({}, {}) 右下角({}, {})", 
        prep.top_left_block.0, prep.top_left_block.1,
        prep.bottom_right_block.0, prep.bottom_right_block.1);
    println!("扫描范围 - 世界坐标: 左上角x:[{},{}) z:[{},{}) 右下角x:[{},{}) z:[{},{})", 
        prep.top_left_world.0, prep.top_left_world.0 + 16,
        prep.top_left_world.1, prep.top_left_world.1 + 16,
        prep.bottom_right_world.0, prep.bottom_right_world.0 + 16,
        prep.bottom_right_world.1, prep.bottom_right_world.1 + 16);
    println!("区块总数: {}×{}={}", prep.x_count, prep.z_count, prep.total_blocks);
    
    match config.mode {
        Mode::Check => {
            println!("模式: Check");
        }
        Mode::Match => {
            println!("模式: Match");
            if let Some(p) = &prep.pattern {
                println!("匹配模式: {}×{}", p.width, p.height);
                println!("匹配目标: {}", config.match_target);
            }
        }
        Mode::Count => {
            println!("模式: Count");
            println!("计数形状: {}", match config.count_shape {
                CountShape::Square => "正方形",
                CountShape::Circle => "圆形",
            });
            println!("区域尺寸: {}", config.count_size);
            println!("目标数量: {}", config.count_target);
        }
    }
    
    if let Some(limit) = config.memory_limit_gib {
        println!("内存限制: {} GiB", limit);
    }
    
    println!("输出网格文件: {}", config.output_map);
    println!("输出匹配文件: {}", config.output_match);
    println!("输出计数文件: {}", config.output_count);
    println!("输出日志文件: {}", config.output_log);
    println!("终端模式: {:?}", config.terminal_mode);
    println!("================");
}

pub fn print_chunking_info(config: &Config, prep: &PreprocessedData) {
    if !is_full(config) {
        return;
    }
    
    if prep.chunk_count > 1 {
        println!("分块处理: 是");
        println!("分块数量: {}", prep.chunk_count);
        println!("每块大小: {}", prep.chunk_size);
    } else {
        println!("分块处理: 否");
    }
}

pub fn print_grid_start(config: &Config) {
    if is_full(config) {
        println!("开始生成网格...");
    }
}

pub fn print_grid_done(config: &Config) {
    if is_full(config) {
        println!("网格生成完成");
    }
}

pub fn print_match_start(config: &Config) {
    if is_full(config) {
        println!("开始模式匹配...");
    }
}

pub fn print_match_done(config: &Config) {
    if is_full(config) {
        println!("匹配完成");
    }
}

pub fn print_match_file_written(config: &Config, path: &std::path::Path) {
    if is_full(config) {
        println!("匹配结果已写入 {}", path.display());
    }
}

pub fn print_count_start(config: &Config) {
    if is_full(config) {
        println!("开始计数...");
    }
}

pub fn print_count_done(config: &Config) {
    if is_full(config) {
        println!("计数完成");
    }
}

pub fn print_count_file_written(config: &Config, path: &std::path::Path) {
    if is_full(config) {
        println!("计数结果已写入 {}", path.display());
    }
}

pub fn print_grid_file_written(config: &Config, path: &std::path::Path) {
    if is_full(config) {
        println!("网格文件已写入 {}", path.display());
    }
}

pub fn print_stats(
    config: &Config,
    prep_time: Duration,
    grid_time: Duration,
    process_time: Duration,
    output_time: Duration,
    total_time: Duration,
    slime_count: usize,
    total_blocks: usize,
) {
    if !is_enabled(config) {
        return;
    }
    
    println!("\n==== [Stats] ====");
    println!("预处理耗时: {:?}", prep_time);
    println!("网格生成耗时: {:?}", grid_time);
    println!("处理耗时: {:?}", process_time);
    println!("输出耗时: {:?}", output_time);
    println!("总耗时: {:?}", total_time);
    
    let blocks_per_second = if total_time.as_secs_f64() > 0.0 {
        (total_blocks as f64) / total_time.as_secs_f64()
    } else {
        0.0
    };
    println!("处理速度: {:.0} 区块/秒", blocks_per_second);
    println!("史莱姆区块数: {} ({:.2}%)", slime_count, 
        (slime_count as f64 / total_blocks as f64) * 100.0);
    println!("================");
}

pub fn print_match_summary(config: &Config, matches: &[MatchResult]) {
    if !is_full(config) {
        return;
    }
    
    println!("匹配数量: {}", matches.len());
    for m in matches {
        let distance = ((m.distance_sq as f64).sqrt() * 16.0).round() as i64;
        println!(
            "  区块: ({}, {}) | 世界坐标: x:[{},{}) z:[{},{}) | 距离: {}m",
            m.block_x, m.block_z,
            m.world_x, m.world_x + 16,
            m.world_z, m.world_z + 16,
            distance
        );
    }
}

pub fn print_count_summary(config: &Config, results: &[CountResult]) {
    if !is_full(config) {
        return;
    }
    
    println!("计数结果数量: {}", results.len());
    for r in results {
        let distance = ((r.distance_sq as f64).sqrt() * 16.0).round() as i64;
        println!(
            "  中心: ({}, {}) | 世界坐标: x:[{},{}) z:[{},{}) | 史莱姆数: {} | 距离: {}m",
            r.block_x, r.block_z,
            r.world_x, r.world_x + 16,
            r.world_z, r.world_z + 16,
            r.slime_count, distance
        );
    }
}