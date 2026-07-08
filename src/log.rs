use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Duration;

use crate::config::Config;
use crate::preprocess::PreprocessedData;

pub struct LogWriter {
    file: Option<BufWriter<File>>,
    enabled: bool,
}

impl LogWriter {
    pub fn new(config: &Config) -> std::io::Result<Self> {
        if !config.output_log {
            return Ok(LogWriter { file: None, enabled: false });
        }
        
        let filename = format!("{}_log.txt", config.world_seed);
        let path = Path::new(&filename);
        let file = BufWriter::new(File::create(path)?);
        
        Ok(LogWriter { file: Some(file), enabled: true })
    }
    
    pub fn write(&mut self, message: &str) -> std::io::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let file = self.file.as_mut().unwrap();
        writeln!(file, "[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), message)
    }
    
    pub fn write_info(&mut self, config: &Config, prep: &PreprocessedData) -> std::io::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let file = self.file.as_mut().unwrap();
        
        writeln!(file, "[Info]")?;
        writeln!(file, "种子: {}", config.world_seed)?;
        writeln!(file, "中心区块: ({}, {})", prep.center_block_x, prep.center_block_z)?;
        writeln!(file, "中心世界坐标: x:[{},{}) z:[{},{})", 
            prep.center_world_x, prep.center_world_x + 16,
            prep.center_world_z, prep.center_world_z + 16)?;
        writeln!(file, "扫描半径: {}", config.radius)?;
        writeln!(file, "扫描范围 - 区块: 左上角({}, {}) 右下角({}, {})", 
            prep.top_left_block.0, prep.top_left_block.1,
            prep.bottom_right_block.0, prep.bottom_right_block.1)?;
        writeln!(file, "扫描范围 - 世界坐标: 左上角x:[{},{}) z:[{},{}) 右下角x:[{},{}) z:[{},{})", 
            prep.top_left_world.0, prep.top_left_world.0 + 16,
            prep.top_left_world.1, prep.top_left_world.1 + 16,
            prep.bottom_right_world.0, prep.bottom_right_world.0 + 16,
            prep.bottom_right_world.1, prep.bottom_right_world.1 + 16)?;
        writeln!(file, "区块总数: {}×{}={}", prep.x_count, prep.z_count, prep.total_blocks)?;
        
        match config.mode {
            crate::config::Mode::Check => {
                writeln!(file, "模式: Check")?;
            }
            crate::config::Mode::Match => {
                writeln!(file, "模式: Match")?;
                if let Some(p) = &prep.pattern {
                    writeln!(file, "匹配图案: {}×{}", p.width, p.height)?;
                    writeln!(file, "匹配目标: {}", config.match_target)?;
                }
            }
            crate::config::Mode::Count => {
                writeln!(file, "模式: Count")?;
                writeln!(file, "计数形状: {}", match config.count_shape {
                    crate::config::CountShape::Square => "正方形",
                    crate::config::CountShape::Circle => "圆形",
                })?;
                writeln!(file, "区域尺寸: {}", config.count_size)?;
                writeln!(file, "目标数量: {}", config.count_target)?;
            }
        }
        
        if let Some(limit) = config.memory_limit_gib {
            writeln!(file, "内存限制: {} GiB", limit)?;
        }
        
        writeln!(file)?;
        
        Ok(())
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
    ) -> std::io::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let file = self.file.as_mut().unwrap();
        
        writeln!(file, "[Stats]")?;
        writeln!(file, "预处理耗时: {:?}", prep_time)?;
        writeln!(file, "网格生成耗时: {:?}", grid_time)?;
        writeln!(file, "处理耗时: {:?}", process_time)?;
        writeln!(file, "输出耗时: {:?}", output_time)?;
        writeln!(file, "总耗时: {:?}", total_time)?;
        
        let blocks_per_second = if total_time.as_secs_f64() > 0.0 {
            (total_blocks as f64) / total_time.as_secs_f64()
        } else {
            0.0
        };
        writeln!(file, "处理速度: {:.0} 区块/秒", blocks_per_second)?;
        writeln!(file, "史莱姆区块数: {} ({:.2}%)", slime_count, 
            (slime_count as f64 / total_blocks as f64) * 100.0)?;
        
        Ok(())
    }
    
    pub fn flush(&mut self) -> std::io::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        self.file.as_mut().unwrap().flush()
    }
}
