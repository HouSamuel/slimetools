use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::config::Config;
use crate::preprocess::PreprocessedData;
use crate::matcher::MatchResult;

pub fn output_match_file(
    config: &Config,
    prep: &PreprocessedData,
    matches: &[MatchResult],
) -> std::io::Result<std::path::PathBuf> {
    let filename = format!("{}_match.txt", config.world_seed);
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
    
    if let Some(p) = &prep.pattern {
        writeln!(file, "匹配模式: {}×{}", p.width, p.height)?;
        writeln!(file, "匹配图案:")?;
        for i in 0..p.height {
            let start = i * p.width;
            let end = start + p.width;
            let row: Vec<String> = p.data[start..end].iter().map(|v| format!("{}", v)).collect();
            writeln!(file, "  {}", row.join(", "))?;
        }
    }
    
    writeln!(file, "匹配目标: {}", config.match_target)?;
    writeln!(file, "找到数量: {}", matches.len())?;
    
    if let Some(limit) = config.memory_limit_gib {
        writeln!(file, "内存限制: {} GiB", limit)?;
    }
    
    writeln!(file)?;
    writeln!(file, "[Output]")?;
    
    for m in matches {
        let distance = ((m.distance_sq as f64).sqrt() * 16.0).round() as i64;
        writeln!(file, 
            "区块: ({}, {}) | 世界坐标: x:[{},{}) z:[{},{}) | 距离中心: {}m",
            m.block_x, m.block_z,
            m.world_x, m.world_x + 16,
            m.world_z, m.world_z + 16,
            distance
        )?;
    }
    
    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Mode};

    #[test]
    fn test_output_match_file() -> std::io::Result<()> {
        let config = Config {
            world_seed: 20260627,
            center_block_x: 0,
            center_block_z: 0,
            radius: 5,
            mode: Mode::Match,
            memory_limit_gib: None,
            pattern: Some(&[&[2, 2], &[2, 2]]),
            match_target: 0,
            count_shape: crate::config::CountShape::Square,
            count_size: 5,
            count_target: 10,
            output_map: false,
            output_match: true,
            output_count: false,
            output_log: true,
            terminal_output: true,
        };
        
        let prep = crate::preprocess::preprocess(&config).unwrap();
        let grid = crate::grid::generate_grid(&config, &prep);
        let matches = crate::matcher::find_matches(&config, &prep, &grid);
        
        let path = output_match_file(&config, &prep, &matches)?;
        
        assert!(path.exists());
        std::fs::remove_file(path)?;
        
        Ok(())
    }
}