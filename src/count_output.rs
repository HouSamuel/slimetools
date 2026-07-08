use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::config::{Config, CountShape};
use crate::preprocess::PreprocessedData;
use crate::counter::CountResult;

pub fn output_count_file(
    config: &Config,
    prep: &PreprocessedData,
    results: &[CountResult],
) -> std::io::Result<std::path::PathBuf> {
    let filename = format!("{}_count.txt", config.world_seed);
    let path = Path::new(&filename);
    
    let mut file = BufWriter::new(File::create(path)?);
    
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
    writeln!(file, "计数模式: {}", match config.count_shape {
        CountShape::Square => "正方形",
        CountShape::Circle => "圆形",
    })?;
    writeln!(file, "区域尺寸: {}", config.count_size)?;
    writeln!(file, "目标数量: {}", config.count_target)?;
    writeln!(file, "找到数量: {}", results.len())?;
    
    if config.memory_limit_gib > 0.0 {
        writeln!(file, "内存限制: {} GiB", config.memory_limit_gib)?;
    } else {
        writeln!(file, "内存限制: 无限制")?;
    }
    
    writeln!(file)?;
    writeln!(file, "[Output]")?;
    
    for result in results {
        let distance = ((result.distance_sq as f64).sqrt() * 16.0).round() as i64;
        writeln!(file, 
            "中心区块: ({}, {}) | 世界坐标: x:[{},{}) z:[{},{}) | 史莱姆区块数: {} | 距离中心: {}m",
            result.block_x, result.block_z,
            result.world_x, result.world_x + 16,
            result.world_z, result.world_z + 16,
            result.slime_count, distance
        )?;
    }
    
    Ok(path.to_path_buf())
}
