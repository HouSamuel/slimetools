use crate::config::{Config, TerminalMode};
use crate::preprocess::PreprocessedData;
use crate::matcher::MatchResult;
use crate::counter::CountResult;
use crate::info;
use crate::CONFIG;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn is_enabled() -> bool {
    CONFIG.terminal_mode != TerminalMode::None
}

fn is_full() -> bool {
    CONFIG.terminal_mode == TerminalMode::Full
}

pub fn print_info(config: &Config, prep: &PreprocessedData) {
    if !is_enabled() {
        return;
    }
    
    println!("{}", info::build_info(config, prep));
}

pub struct ProgressDisplay {
    chunk_count: usize,
    chunk_rows: usize,
    current_chunk: usize,
    grid_elapsed: Duration,
    last_grid_percent: f64,
    last_process_percent: f64,
    update_interval: f64,
}

impl ProgressDisplay {
    pub fn new(chunk_count: usize, chunk_rows: usize, update_interval: f64) -> Self {
        ProgressDisplay {
            chunk_count,
            chunk_rows,
            current_chunk: 0,
            grid_elapsed: Duration::from_secs(0),
            last_grid_percent: -1.0,
            last_process_percent: -1.0,
            update_interval,
        }
    }

    pub fn print_header(&mut self, prep: &PreprocessedData) {
        if !is_enabled() {
            return;
        }

        println!("==== [Progress] ====");
        
        let total_memory_gib = (prep.total_blocks as f64) / (1024.0 * 1024.0 * 1024.0);
        println!("  预估内存: {:.4} GiB", total_memory_gib);
        
        if self.chunk_count > 1 {
            println!("  分块处理: 是");
            println!("  分块数量: {}", self.chunk_count);
            println!("  每块行数: {}", self.chunk_rows);
            let chunk_memory_gib = ((self.chunk_rows * prep.x_count) as f64) / (1024.0 * 1024.0 * 1024.0);
            println!("  分块内存: {:.4} GiB", chunk_memory_gib);
        } else {
            println!("  分块处理: 否");
        }
        
        println!("  开始计算...");
    }

    pub fn start_chunk(&mut self, chunk_idx: usize) {
        if !is_enabled() {
            return;
        }

        self.current_chunk = chunk_idx;
        self.grid_elapsed = Duration::from_secs(0);
        self.last_grid_percent = -1.0;
        self.last_process_percent = -1.0;
        print!("[{}/{}]       生成网格    0%     0s    处理网格    0%     0s", 
            chunk_idx + 1, self.chunk_count);
        io::stdout().flush().unwrap();
    }

    pub fn update_grid(&mut self, percent: f64, elapsed: Duration) {
        if !is_enabled() {
            return;
        }

        if (percent - self.last_grid_percent).abs() < self.update_interval && percent < 100.0 {
            return;
        }
        self.last_grid_percent = percent;
        
        print!("\r[{}/{}]       生成网格   {:.0}%     {}s    处理网格    0%     0s", 
            self.current_chunk + 1, self.chunk_count, percent, elapsed.as_secs());
        
        io::stdout().flush().unwrap();
    }

    pub fn finish_grid(&mut self, elapsed: Duration) {
        if !is_enabled() {
            return;
        }

        self.grid_elapsed = elapsed;
        
        print!("\r[{}/{}]       生成网格  100%     {}s    处理网格    0%     0s", 
            self.current_chunk + 1, self.chunk_count, elapsed.as_secs());
        
        io::stdout().flush().unwrap();
    }

    pub fn update_process(&mut self, percent: f64, elapsed: Duration) {
        if !is_enabled() {
            return;
        }

        if (percent - self.last_process_percent).abs() < self.update_interval && percent < 100.0 {
            return;
        }
        self.last_process_percent = percent;
        
        print!("\r[{}/{}]       生成网格  100%     {}s    处理网格   {:.0}%     {}s", 
            self.current_chunk + 1, self.chunk_count, self.grid_elapsed.as_secs(), percent, elapsed.as_secs());
        
        io::stdout().flush().unwrap();
    }

    pub fn finish_process(&mut self, elapsed: Duration) {
        if !is_enabled() {
            return;
        }

        println!("\r[{}/{}]       生成网格  100%     {}s    处理网格  100%     {}s", 
            self.current_chunk + 1, self.chunk_count, self.grid_elapsed.as_secs(), elapsed.as_secs());
        
        io::stdout().flush().unwrap();
    }

    pub fn finish_all(&self) {}
}

pub type SharedProgressDisplay = Arc<Mutex<ProgressDisplay>>;

pub fn print_match_start(_config: &Config) {
    if is_enabled() {
        println!("[Info] 开始模式匹配...");
    }
}

pub fn print_match_done(_config: &Config) {
    if is_enabled() {
        println!("[Info] 匹配完成");
    }
}

pub fn print_match_file_written(_config: &Config, path: &std::path::Path) {
    if is_full() {
        println!("[Info] 匹配结果已写入 {}", path.display());
    }
}

pub fn print_count_start(_config: &Config) {
    if is_enabled() {
        println!("[Info] 开始计数...");
    }
}

pub fn print_count_done(_config: &Config) {
    if is_enabled() {
        println!("[Info] 计数完成");
    }
}

pub fn print_count_file_written(_config: &Config, path: &std::path::Path) {
    if is_full() {
        println!("[Info] 计数结果已写入 {}", path.display());
    }
}

pub fn print_grid_file_written(_config: &Config, path: &std::path::Path) {
    if is_full() {
        println!("[Info] 网格文件已写入 {}", path.display());
    }
}

pub fn print_stats(
    _config: &Config,
    prep_time: Duration,
    grid_time: Duration,
    process_time: Duration,
    output_time: Duration,
    total_time: Duration,
    slime_count: usize,
    total_blocks: usize,
) {
    if !is_enabled() {
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

pub fn print_match_summary(_config: &Config, matches: &[MatchResult]) {
    if !is_full() {
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

pub fn print_count_summary(_config: &Config, results: &[CountResult]) {
    if !is_full() {
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
