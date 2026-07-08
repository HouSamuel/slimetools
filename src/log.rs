use crate::config::{Config, TerminalMode};
use crate::preprocess::PreprocessedData;
use crate::matcher::MatchResult;
use crate::counter::CountResult;
use crate::info;
use crate::CONFIG;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

fn is_enabled(config: &Config) -> bool {
    config.output_log
}

fn is_full(config: &Config) -> bool {
    config.terminal_mode == TerminalMode::Full
}

fn get_timestamp() -> String {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let secs = since_epoch.as_secs();
    let nanos = since_epoch.subsec_nanos();
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
        (secs / 31536000 + 1970),
        ((secs / 86400) % 365 / 30 + 1),
        (secs / 86400 % 30 + 1),
        (secs / 3600) % 24,
        (secs / 60) % 60,
        secs % 60,
        nanos / 1_000_000
    )
}

pub struct LogWriterInner {
    writer: BufWriter<File>,
    enabled: bool,
}

impl LogWriterInner {
    fn new(config: &Config) -> Option<Self> {
        if !is_enabled(config) {
            return None;
        }
        
        let log_dir = Path::new("logs");
        if let Err(e) = fs::create_dir_all(log_dir) {
            eprintln!("[Error] 创建日志目录失败: {}", e);
            return None;
        }
        
        let log_path = log_dir.join(format!("slime_scan_{}.log", 
            chrono::Local::now().format("%Y%m%d_%H%M%S")));
        
        match File::create(&log_path) {
            Ok(file) => {
                let mut writer = BufWriter::new(file);
                let _ = writeln!(writer, "[{}] 日志文件创建: {}", get_timestamp(), log_path.display());
                Some(Self { writer, enabled: true })
            }
            Err(e) => {
                eprintln!("[Error] 创建日志文件失败: {}", e);
                None
            }
        }
    }

    fn write_info(&mut self, config: &Config, prep: &PreprocessedData) {
        if !self.enabled {
            return;
        }
        
        let info_str = info::build_info(config, prep);
        let _ = writeln!(self.writer, "[{}]\n{}", get_timestamp(), info_str);
        let _ = self.writer.flush();
    }

    fn write_progress_header(&mut self, config: &Config, prep: &PreprocessedData) {
        if !self.enabled {
            return;
        }

        let _ = writeln!(self.writer, "[{}] ==== [Progress] ====", get_timestamp());
        
        let total_memory_gib = (prep.total_blocks as f64) / (1024.0 * 1024.0 * 1024.0);
        let _ = writeln!(self.writer, "[{}]   预估内存: {:.4} GiB", get_timestamp(), total_memory_gib);
        
        if prep.chunk_count > 1 {
            let _ = writeln!(self.writer, "[{}]   分块处理: 是", get_timestamp());
            let _ = writeln!(self.writer, "[{}]   分块数量: {}", get_timestamp(), prep.chunk_count);
            let chunk_rows = prep.chunk_size / prep.x_count;
            let _ = writeln!(self.writer, "[{}]   每块行数: {}", get_timestamp(), chunk_rows);
            let chunk_memory_gib = (prep.chunk_size as f64) / (1024.0 * 1024.0 * 1024.0);
            let _ = writeln!(self.writer, "[{}]   分块内存: {:.4} GiB", get_timestamp(), chunk_memory_gib);
        } else {
            let _ = writeln!(self.writer, "[{}]   分块处理: 否", get_timestamp());
        }
        
        let _ = writeln!(self.writer, "[{}]   开始计算...", get_timestamp());
        let _ = writeln!(self.writer, "[{}]   进度: 0.0%", get_timestamp());
        let _ = self.writer.flush();
    }

    fn write_chunk_start(&mut self, chunk_idx: usize, chunk_count: usize) {
        if !self.enabled {
            return;
        }
        
        let _ = writeln!(self.writer, "[{}] [Progress] [{}/{}]     0%       0s", 
            get_timestamp(), chunk_idx + 1, chunk_count);
        let _ = self.writer.flush();
    }

    fn write_chunk_update(&mut self, chunk_idx: usize, chunk_count: usize, percent: f64, elapsed: Duration) {
        if !self.enabled {
            return;
        }
        
        let total_percent = ((chunk_idx as f64 + percent / 100.0) / chunk_count as f64) * 100.0;
        let _ = writeln!(self.writer, "[{}] [Progress] [{}/{}]     {:.1}%       {}s | 总进度: {:.1}%", 
            get_timestamp(), chunk_idx + 1, chunk_count, percent, elapsed.as_secs(), total_percent);
        let _ = self.writer.flush();
    }

    fn write_chunk_finish(&mut self, chunk_idx: usize, chunk_count: usize, elapsed: Duration) {
        if !self.enabled {
            return;
        }
        
        let total_percent = ((chunk_idx + 1) as f64 / chunk_count as f64) * 100.0;
        let _ = writeln!(self.writer, "[{}] [Progress] [{}/{}]     100%       {}s | 总进度: {:.1}%", 
            get_timestamp(), chunk_idx + 1, chunk_count, elapsed.as_secs(), total_percent);
        let _ = self.writer.flush();
    }

    fn write_progress_finish(&mut self) {
        if !self.enabled {
            return;
        }
        
        let _ = writeln!(self.writer, "[{}] ====================", get_timestamp());
        let _ = self.writer.flush();
    }

    fn write_log(&mut self, message: &str) {
        if !self.enabled {
            return;
        }
        
        let _ = writeln!(self.writer, "[{}] {}", get_timestamp(), message);
        let _ = self.writer.flush();
    }

    fn write_match_start(&mut self) {
        if !self.enabled {
            return;
        }
        
        let _ = writeln!(self.writer, "[{}] [Info] 开始模式匹配...", get_timestamp());
        let _ = self.writer.flush();
    }

    fn write_match_done(&mut self) {
        if !self.enabled {
            return;
        }
        
        let _ = writeln!(self.writer, "[{}] [Info] 匹配完成", get_timestamp());
        let _ = self.writer.flush();
    }

    fn write_match_file_written(&mut self, path: &Path) {
        if !is_full(&CONFIG) {
            return;
        }
        
        let _ = writeln!(self.writer, "[{}] [Info] 匹配结果已写入 {}", get_timestamp(), path.display());
        let _ = self.writer.flush();
    }

    fn write_count_start(&mut self) {
        if !self.enabled {
            return;
        }
        
        let _ = writeln!(self.writer, "[{}] [Info] 开始计数...", get_timestamp());
        let _ = self.writer.flush();
    }

    fn write_count_done(&mut self) {
        if !self.enabled {
            return;
        }
        
        let _ = writeln!(self.writer, "[{}] [Info] 计数完成", get_timestamp());
        let _ = self.writer.flush();
    }

    fn write_count_file_written(&mut self, path: &Path) {
        if !is_full(&CONFIG) {
            return;
        }
        
        let _ = writeln!(self.writer, "[{}] [Info] 计数结果已写入 {}", get_timestamp(), path.display());
        let _ = self.writer.flush();
    }

    fn write_grid_file_written(&mut self, path: &Path) {
        if !is_full(&CONFIG) {
            return;
        }
        
        let _ = writeln!(self.writer, "[{}] [Info] 网格文件已写入 {}", get_timestamp(), path.display());
        let _ = self.writer.flush();
    }

    fn write_stats(
        &mut self,
        prep_time: Duration,
        grid_time: Duration,
        process_time: Duration,
        output_time: Duration,
        total_time: Duration,
        slime_count: usize,
        total_blocks: usize,
    ) {
        if !self.enabled {
            return;
        }
        
        let _ = writeln!(self.writer, "[{}]", get_timestamp());
        let _ = writeln!(self.writer, "==== [Stats] ====");
        let _ = writeln!(self.writer, "预处理耗时: {:?}", prep_time);
        let _ = writeln!(self.writer, "网格生成耗时: {:?}", grid_time);
        let _ = writeln!(self.writer, "处理耗时: {:?}", process_time);
        let _ = writeln!(self.writer, "输出耗时: {:?}", output_time);
        let _ = writeln!(self.writer, "总耗时: {:?}", total_time);
        
        let blocks_per_second = if total_time.as_secs_f64() > 0.0 {
            (total_blocks as f64) / total_time.as_secs_f64()
        } else {
            0.0
        };
        let _ = writeln!(self.writer, "处理速度: {:.0} 区块/秒", blocks_per_second);
        let _ = writeln!(self.writer, "史莱姆区块数: {} ({:.2}%)", slime_count, 
            (slime_count as f64 / total_blocks as f64) * 100.0);
        let _ = writeln!(self.writer, "================");
        let _ = self.writer.flush();
    }

    fn write_match_summary(&mut self, matches: &[MatchResult]) {
        if !is_full(&CONFIG) {
            return;
        }
        
        let _ = writeln!(self.writer, "[{}]", get_timestamp());
        let _ = writeln!(self.writer, "匹配数量: {}", matches.len());
        for m in matches {
            let distance = ((m.distance_sq as f64).sqrt() * 16.0).round() as i64;
            let _ = writeln!(self.writer,
                "  区块: ({}, {}) | 世界坐标: x:[{},{}) z:[{},{}) | 距离: {}m",
                m.block_x, m.block_z,
                m.world_x, m.world_x + 16,
                m.world_z, m.world_z + 16,
                distance
            );
        }
        let _ = self.writer.flush();
    }

    fn write_count_summary(&mut self, results: &[CountResult]) {
        if !is_full(&CONFIG) {
            return;
        }
        
        let _ = writeln!(self.writer, "[{}]", get_timestamp());
        let _ = writeln!(self.writer, "计数结果数量: {}", results.len());
        for r in results {
            let distance = ((r.distance_sq as f64).sqrt() * 16.0).round() as i64;
            let _ = writeln!(self.writer,
                "  中心: ({}, {}) | 世界坐标: x:[{},{}) z:[{},{}) | 史莱姆数: {} | 距离: {}m",
                r.block_x, r.block_z,
                r.world_x, r.world_x + 16,
                r.world_z, r.world_z + 16,
                r.slime_count, distance
            );
        }
        let _ = self.writer.flush();
    }
}

pub struct LogWriter {
    pub inner: Arc<Mutex<Option<LogWriterInner>>>,
}

impl LogWriter {
    pub fn new(config: &Config) -> Option<Self> {
        let inner = LogWriterInner::new(config);
        Some(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    pub fn write_info(&mut self, config: &Config, prep: &PreprocessedData) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_info(config, prep);
            }
        }
    }

    pub fn write_progress_header(&mut self, config: &Config, prep: &PreprocessedData) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_progress_header(config, prep);
            }
        }
    }

    pub fn write_chunk_start(&mut self, chunk_idx: usize, chunk_count: usize) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_chunk_start(chunk_idx, chunk_count);
            }
        }
    }

    pub fn write_chunk_update(&mut self, chunk_idx: usize, chunk_count: usize, percent: f64, elapsed: Duration) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_chunk_update(chunk_idx, chunk_count, percent, elapsed);
            }
        }
    }

    pub fn write_chunk_finish(&mut self, chunk_idx: usize, chunk_count: usize, elapsed: Duration) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_chunk_finish(chunk_idx, chunk_count, elapsed);
            }
        }
    }

    pub fn write_progress_finish(&mut self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_progress_finish();
            }
        }
    }

    pub fn write_log(&mut self, message: &str) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_log(message);
            }
        }
    }

    pub fn write_match_start(&mut self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_match_start();
            }
        }
    }

    pub fn write_match_done(&mut self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_match_done();
            }
        }
    }

    pub fn write_match_file_written(&mut self, path: &Path) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_match_file_written(path);
            }
        }
    }

    pub fn write_count_start(&mut self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_count_start();
            }
        }
    }

    pub fn write_count_done(&mut self) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_count_done();
            }
        }
    }

    pub fn write_count_file_written(&mut self, path: &Path) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_count_file_written(path);
            }
        }
    }

    pub fn write_grid_file_written(&mut self, path: &Path) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_grid_file_written(path);
            }
        }
    }

    pub fn write_stats(
        &mut self,
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
                inner.write_stats(prep_time, grid_time, process_time, output_time, total_time, slime_count, total_blocks);
            }
        }
    }

    pub fn write_match_summary(&mut self, matches: &[MatchResult]) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_match_summary(matches);
            }
        }
    }

    pub fn write_count_summary(&mut self, results: &[CountResult]) {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(inner) = guard.as_mut() {
                inner.write_count_summary(results);
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
