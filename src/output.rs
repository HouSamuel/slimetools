use crate::config::{Config, TerminalMode};
use crate::preprocess::PreprocessedData;
use crate::matcher::MatchResult;
use crate::info;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn is_terminal_enabled(config: &Config) -> bool {
    config.terminal_mode != TerminalMode::None
}

fn is_terminal_full(config: &Config) -> bool {
    config.terminal_mode == TerminalMode::Full
}

fn get_timestamp() -> String {
    chrono::Local::now().format("%Y%m%d_%H%M%S").to_string()
}

fn get_timestamp_with_ms() -> String {
    chrono::Local::now().format("[%Y-%m-%d %H:%M:%S%.3f]").to_string()
}

pub fn build_info(config: &Config, prep: &PreprocessedData) -> String {
    info::build_info(config, prep)
}

pub fn print_info(config: &Config, prep: &PreprocessedData) {
    if !is_terminal_enabled(config) {
        return;
    }
    println!("{}", build_info(config, prep));
}

pub struct LogWriter {
    inner: Arc<Mutex<Option<LogWriterInner>>>,
}

struct LogWriterInner {
    writer: BufWriter<File>,
    enabled: bool,
}

impl LogWriterInner {
    fn new(config: &Config) -> Option<Self> {
        if !config.output_log {
            return None;
        }
        
        let timestamp_str = get_timestamp();
        let log_filename = format!("{}_log_{}.log", config.world_seed, timestamp_str);
        let log_path = Path::new(&log_filename);
        
        match File::create(&log_path) {
            Ok(file) => {
                let mut writer = BufWriter::new(file);
                let _ = writeln!(writer, "日志文件创建: {}", log_path.display());
                Some(Self { writer, enabled: true })
            }
            Err(e) => {
                eprintln!("[Error] 创建日志文件失败: {}", e);
                None
            }
        }
    }

    fn write(&mut self, message: &str) {
        if !self.enabled {
            return;
        }
        let _ = writeln!(self.writer, "{}{}", get_timestamp_with_ms(), message);
        let _ = self.writer.flush();
    }

    fn write_without_timestamp(&mut self, message: &str) {
        if !self.enabled {
            return;
        }
        let _ = writeln!(self.writer, "{}", message);
        let _ = self.writer.flush();
    }
}

impl LogWriter {
    pub fn new(config: &Config) -> Self {
        let inner = LogWriterInner::new(config);
        LogWriter {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    pub fn write_info(&self, config: &Config, prep: &PreprocessedData) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_without_timestamp(&build_info(config, prep));
            }
        }
    }

    pub fn write_progress_header(&self, config: &Config, prep: &PreprocessedData) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                let mut progress_header = String::new();
                progress_header.push_str("==== [Progress] ====\n");
                
                let total_memory_gib = (prep.total_blocks as f64) / (1024.0 * 1024.0 * 1024.0);
                progress_header.push_str(&format!("  预估内存: {:.4} GiB\n", total_memory_gib));
                
                if prep.chunk_count > 1 {
                    progress_header.push_str("  分块处理: 是\n");
                    progress_header.push_str(&format!("  分块数量: {}\n", prep.chunk_count));
                    let chunk_rows = prep.chunk_size / prep.x_count;
                    progress_header.push_str(&format!("  每块行数: {}\n", chunk_rows));
                    let chunk_memory_gib = (prep.chunk_size as f64) / (1024.0 * 1024.0 * 1024.0);
                    progress_header.push_str(&format!("  分块内存: {:.4} GiB\n", chunk_memory_gib));
                } else {
                    progress_header.push_str("  分块处理: 否\n");
                }
                
                progress_header.push_str("  开始计算...\n");
                inner.write_without_timestamp(&progress_header);
            }
        }
    }

    pub fn write_progress_update(&self, message: &str) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write(message);
            }
        }
    }

    pub fn write_progress_finish(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_without_timestamp("====================");
            }
        }
    }

    pub fn write_log(&self, message: &str) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write(message);
            }
        }
    }

    pub fn write_file_written(&self, file_type: &str, path: &Path) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write(&format!("[Info] {}文件已写入 {}", file_type, path.display()));
            }
        }
    }

    pub fn write_stats(
        &self,
        prep_time: Duration,
        grid_time: Duration,
        process_time: Duration,
        output_time: Duration,
        total_time: Duration,
        slime_count: usize,
        total_blocks: usize,
    ) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                let mut stats = String::new();
                stats.push_str("==== [Stats] ====\n");
                stats.push_str(&format!("预处理耗时: {:?}\n", prep_time));
                stats.push_str(&format!("网格生成耗时: {:?}\n", grid_time));
                stats.push_str(&format!("处理耗时: {:?}\n", process_time));
                stats.push_str(&format!("输出耗时: {:?}\n", output_time));
                stats.push_str(&format!("总耗时: {:?}\n", total_time));
                
                let blocks_per_second = if total_time.as_secs_f64() > 0.0 {
                    (total_blocks as f64) / total_time.as_secs_f64()
                } else {
                    0.0
                };
                stats.push_str(&format!("处理速度: {:.0} 区块/秒\n", blocks_per_second));
                stats.push_str(&format!("史莱姆区块数: {} ({:.2}%)\n", slime_count, 
                    (slime_count as f64 / total_blocks as f64) * 100.0));
                stats.push_str("================");
                inner.write_without_timestamp(&stats);
            }
        }
    }

    pub fn write_check_summary(&self, slime_count: usize, total_blocks: usize) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                let mut summary = String::new();
                summary.push_str("==== [Result] ====\n");
                summary.push_str(&format!("史莱姆区块数: {} ({:.2}%)\n", slime_count, 
                    (slime_count as f64 / total_blocks as f64) * 100.0));
                summary.push_str("================");
                inner.write_without_timestamp(&summary);
            }
        }
    }

    pub fn write_match_summary(&self, matches: &[MatchResult]) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                let mut summary = String::new();
                summary.push_str("==== [Result] ====\n");
                summary.push_str(&format!("匹配数量: {}\n", matches.len()));
                for m in matches {
                    let distance = ((m.distance_sq as f64).sqrt() * 16.0).round() as i64;
                    summary.push_str(&format!(
                        "  区块: ({}, {}) | 世界坐标: x:[{},{}) z:[{},{}) | 距离: {}m\n",
                        m.block_x, m.block_z,
                        m.world_x, m.world_x + 16,
                        m.world_z, m.world_z + 16,
                        distance
                    ));
                }
                summary.push_str("================");
                inner.write_without_timestamp(&summary);
            }
        }
    }
}

impl Clone for LogWriter {
    fn clone(&self) -> Self {
        LogWriter {
            inner: Arc::clone(&self.inner),
        }
    }
}

pub struct ProgressDisplay {
    chunk_count: usize,
    chunk_rows: usize,
    current_chunk: usize,
    grid_elapsed: Duration,
    last_grid_percent: f64,
    last_process_percent: f64,
    update_interval: f64,
    start_time: Instant,
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
            start_time: Instant::now(),
        }
    }

    pub fn print_header(&mut self, config: &Config, prep: &PreprocessedData) {
        if !is_terminal_enabled(config) {
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
        self.start_time = Instant::now();
    }

    pub fn start_chunk(&mut self, config: &Config, chunk_idx: usize) {
        if !is_terminal_enabled(config) {
            return;
        }

        self.current_chunk = chunk_idx;
        self.grid_elapsed = Duration::from_secs(0);
        self.last_grid_percent = -1.0;
        self.last_process_percent = -1.0;
        print!("[{}/{}]       生成网格    0%     0s    处理网格    0%     0s", 
            chunk_idx + 1, self.chunk_count);
        std::io::stdout().flush().unwrap();
    }

    pub fn update_grid(&mut self, config: &Config, percent: f64, elapsed: Duration) {
        if !is_terminal_enabled(config) {
            return;
        }

        if (percent - self.last_grid_percent).abs() < self.update_interval && percent < 100.0 {
            return;
        }
        self.last_grid_percent = percent;
        
        print!("\r[{}/{}]       生成网格   {:.0}%     {}s    处理网格    0%     0s", 
            self.current_chunk + 1, self.chunk_count, percent, elapsed.as_secs());
        
        std::io::stdout().flush().unwrap();
    }

    pub fn finish_grid(&mut self, config: &Config, elapsed: Duration) {
        if !is_terminal_enabled(config) {
            return;
        }

        self.grid_elapsed = elapsed;
        
        print!("\r[{}/{}]       生成网格  100%     {}s    处理网格    0%     0s", 
            self.current_chunk + 1, self.chunk_count, elapsed.as_secs());
        
        std::io::stdout().flush().unwrap();
    }

    pub fn update_process(&mut self, config: &Config, percent: f64, elapsed: Duration) {
        if !is_terminal_enabled(config) {
            return;
        }

        if (percent - self.last_process_percent).abs() < self.update_interval && percent < 100.0 {
            return;
        }
        self.last_process_percent = percent;
        
        print!("\r[{}/{}]       生成网格  100%     {}s    处理网格   {:.0}%     {}s", 
            self.current_chunk + 1, self.chunk_count, self.grid_elapsed.as_secs(), percent, elapsed.as_secs());
        
        std::io::stdout().flush().unwrap();
    }

    pub fn finish_process(&mut self, config: &Config, elapsed: Duration) {
        if !is_terminal_enabled(config) {
            return;
        }

        println!("\r[{}/{}]       生成网格  100%     {}s    处理网格  100%     {}s", 
            self.current_chunk + 1, self.chunk_count, self.grid_elapsed.as_secs(), elapsed.as_secs());
        
        std::io::stdout().flush().unwrap();
        
        if self.current_chunk == 0 && self.chunk_count > 1 {
            let total_elapsed = self.start_time.elapsed();
            let avg_per_chunk = total_elapsed.as_secs_f64();
            let remaining_chunks = self.chunk_count - 1;
            let estimated_remaining = (avg_per_chunk * remaining_chunks as f64) as u64;
            
            let hours = estimated_remaining / 3600;
            let minutes = (estimated_remaining % 3600) / 60;
            let seconds = estimated_remaining % 60;
            
            if hours > 0 {
                println!("预估剩余时间: {}小时{}分{}秒", hours, minutes, seconds);
            } else if minutes > 0 {
                println!("预估剩余时间: {}分{}秒", minutes, seconds);
            } else {
                println!("预估剩余时间: {}秒", seconds);
            }
        }
    }
}

pub fn print_grid_file_written(config: &Config, path: &Path) {
    if is_terminal_full(config) {
        println!("[Info] 网格文件已写入 {}", path.display());
    }
}

pub fn print_match_file_written(config: &Config, path: &Path) {
    if is_terminal_full(config) {
        println!("[Info] 匹配结果已写入 {}", path.display());
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
    if !is_terminal_enabled(config) {
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

pub fn print_check_summary(config: &Config, slime_count: usize, total_blocks: usize) {
    if !is_terminal_full(config) {
        return;
    }
    
    println!("\n==== [Result] ====");
    println!("史莱姆区块数: {} ({:.2}%)", slime_count, 
        (slime_count as f64 / total_blocks as f64) * 100.0);
    println!("================");
}

pub fn print_match_summary(config: &Config, matches: &[MatchResult]) {
    if !is_terminal_full(config) {
        return;
    }
    
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

pub fn output_grid_file(config: &Config, prep: &PreprocessedData, grid: &[u8]) -> std::io::Result<PathBuf> {
    crate::grid_output::output_grid_file(config, prep, grid)
}

pub fn output_match_file(config: &Config, prep: &PreprocessedData, matches: &[MatchResult]) -> std::io::Result<PathBuf> {
    crate::match_output::output_match_file(config, prep, matches)
}