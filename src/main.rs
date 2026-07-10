mod config;
mod slime_chunk;
mod preprocess;
mod check;
mod matcher;
mod grid_output;
mod match_output;
mod info;
mod output;

use config::CONFIG;
use std::time::{Instant, Duration};

fn main() -> std::io::Result<()> {
    let config = &CONFIG;
    let start_time = Instant::now();
    
    let prep_start = Instant::now();
    let prep = preprocess::preprocess(config).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let prep_time = prep_start.elapsed();
    
    let log_writer = output::LogWriter::new(config);
    
    if is_terminal_enabled(config) {
        println!("{}", output::build_info(config, &prep));
    }
    
    if config.output_log {
        log_writer.write_info(config, &prep);
        log_writer.write_progress_header(config, &prep);
    }
    
    let mut progress_display = output::ProgressDisplay::new(prep.chunk_count, prep.chunk_size / prep.x_count, config.progress_update_interval);
    if is_terminal_enabled(config) {
        progress_display.print_header(config, &prep);
    }
    
    let grid_start = Instant::now();
    let grid = check::generate_grid(config, &prep);
    let grid_time = grid_start.elapsed();
    
    let slime_count = grid.iter().filter(|&&x| x == 1).count();
    
    let mut process_time = Duration::from_secs(0);
    let mut output_time = Duration::from_secs(0);
    
    match config.mode {
        config::Mode::Check => {
            let output_start = Instant::now();
            
            if config.output_map {
                let path = output::output_grid_file(config, &prep, &grid)?;
                if is_terminal_full(config) {
                    println!("[Info] 网格文件已写入 {}", path.display());
                }
                if config.output_log {
                    log_writer.write_file_written("网格", &path);
                }
            }
            
            if is_terminal_full(config) {
                println!("\n==== [Result] ====");
                println!("史莱姆区块数: {} ({:.2}%)", slime_count, 
                    (slime_count as f64 / prep.total_blocks as f64) * 100.0);
                println!("================");
            }
            
            output_time = output_start.elapsed();
        }
        
        config::Mode::Match => {
            let process_start = Instant::now();
            let matches = matcher::find_matches(config, &prep, &grid);
            process_time = process_start.elapsed();
            
            let output_start = Instant::now();
            
            if config.output_map {
                let path = output::output_grid_file(config, &prep, &grid)?;
                if is_terminal_full(config) {
                    println!("[Info] 网格文件已写入 {}", path.display());
                }
                if config.output_log {
                    log_writer.write_file_written("网格", &path);
                }
            }
            
            if config.output_match {
                let path = output::output_match_file(config, &prep, &matches)?;
                if is_terminal_full(config) {
                    println!("[Info] 匹配结果已写入 {}", path.display());
                }
                if config.output_log {
                    log_writer.write_file_written("匹配", &path);
                }
            }
            
            if is_terminal_full(config) {
                println!("\n==== [Result] ====");
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
                println!("================");
            }
            
            output_time = output_start.elapsed();
        }
    }
    
    let total_time = start_time.elapsed();
    
    if is_terminal_enabled(config) {
        println!("\n==== [Stats] ====");
        println!("预处理耗时: {:?}", prep_time);
        println!("网格生成耗时: {:?}", grid_time);
        println!("处理耗时: {:?}", process_time);
        println!("输出耗时: {:?}", output_time);
        println!("总耗时: {:?}", total_time);
        
        let blocks_per_second = if total_time.as_secs_f64() > 0.0 {
            (prep.total_blocks as f64) / total_time.as_secs_f64()
        } else {
            0.0
        };
        println!("处理速度: {:.0} 区块/秒", blocks_per_second);
        println!("史莱姆区块数: {} ({:.2}%)", slime_count, 
            (slime_count as f64 / prep.total_blocks as f64) * 100.0);
        println!("================");
    }
    
    if config.output_log {
        log_writer.write_stats(prep_time, grid_time, process_time, output_time, total_time, slime_count, prep.total_blocks);
    }
    
    Ok(())
}

fn is_terminal_enabled(config: &config::Config) -> bool {
    config.terminal_mode != config::TerminalMode::None
}

fn is_terminal_full(config: &config::Config) -> bool {
    config.terminal_mode == config::TerminalMode::Full
}