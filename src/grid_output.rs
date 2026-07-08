use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use rayon::prelude::*;

use crate::config::Config;
use crate::preprocess::PreprocessedData;

pub fn build_grid_header(config: &Config, prep: &PreprocessedData) -> String {
    format!(
        "# [Info]\n# 种子: {}\n# 中心区块: ({}, {})\n# 中心世界坐标: x:[{},{}) z:[{},{})\n# 扫描半径: {}\n# 扫描范围 - 区块: 左上角({}, {}) 右下角({}, {})\n# 扫描范围 - 世界坐标: 左上角x:[{},{}) z:[{},{}) 右下角x:[{},{}) z:[{},{})\n# 区块总数: {}×{}={}\n",
        config.world_seed,
        prep.center_block_x, prep.center_block_z,
        prep.center_world_x, prep.center_world_x + 16,
        prep.center_world_z, prep.center_world_z + 16,
        config.radius,
        prep.top_left_block.0, prep.top_left_block.1,
        prep.bottom_right_block.0, prep.bottom_right_block.1,
        prep.top_left_world.0, prep.top_left_world.0 + 16,
        prep.top_left_world.1, prep.top_left_world.1 + 16,
        prep.bottom_right_world.0, prep.bottom_right_world.0 + 16,
        prep.bottom_right_world.1, prep.bottom_right_world.1 + 16,
        prep.x_count, prep.z_count, prep.total_blocks,
    )
}

pub fn output_grid_file(config: &Config, prep: &PreprocessedData, grid: &[u8]) -> std::io::Result<std::path::PathBuf> {
    let filename = format!("{}_map.txt", config.world_seed);
    let path = Path::new(&filename);
    
    let header = build_grid_header(config, prep);
    
    let lines: Vec<String> = (0..prep.z_count)
        .into_par_iter()
        .map(|z_idx| {
            let z_header = String::from_utf8_lossy(&prep.z_headers[z_idx]);
            let cells: String = (0..prep.x_count)
                .map(|x_idx| {
                    let val = grid[z_idx * prep.x_count + x_idx];
                    if val == 1 {
                        String::from_utf8_lossy(&prep.one_cell)
                    } else {
                        String::from_utf8_lossy(&prep.zero_cell)
                    }
                })
                .collect();
            format!("{}{}\n", z_header, cells)
        })
        .collect();
    
    let x_header_line: String = prep.x_headers.iter()
        .map(|h| String::from_utf8_lossy(h))
        .collect();
    let x_header = format!("{}{}\n", " ".repeat(prep.z_col_width), x_header_line);
    
    let mut file = BufWriter::new(File::create(path)?);
    writeln!(file, "{}", header)?;
    write!(file, "{}", x_header)?;
    for line in lines {
        write!(file, "{}", line)?;
    }
    
    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Mode, TerminalMode};

    #[test]
    fn test_build_grid_header() {
        let config = Config {
            world_seed: 20260627,
            center_block_x: 0,
            center_block_z: 0,
            radius: 2,
            mode: Mode::Check,
            memory_limit_gib: None,
            pattern: None,
            match_target: 0,
            count_shape: crate::config::CountShape::Square,
            count_size: 5,
            count_target: 10,
            output_map: true,
            output_match: false,
            output_count: false,
            output_log: true,
            terminal_mode: TerminalMode::Full,
        };
        
        let prep = crate::preprocess::preprocess(&config).unwrap();
        let header = build_grid_header(&config, &prep);
        
        assert!(header.contains("种子"));
        assert!(header.contains("中心区块"));
        assert!(header.contains("扫描半径"));
    }

    #[test]
    fn test_output_grid_file() -> std::io::Result<()> {
        let config = Config {
            world_seed: 20260627,
            center_block_x: 0,
            center_block_z: 0,
            radius: 2,
            mode: Mode::Check,
            memory_limit_gib: None,
            pattern: None,
            match_target: 0,
            count_shape: crate::config::CountShape::Square,
            count_size: 5,
            count_target: 10,
            output_map: true,
            output_match: false,
            output_count: false,
            output_log: true,
            terminal_mode: TerminalMode::Full,
        };
        
        let prep = crate::preprocess::preprocess(&config).unwrap();
        let grid = crate::grid::generate_grid(&config, &prep);
        
        let path = output_grid_file(&config, &prep, &grid)?;
        
        assert!(path.exists());
        std::fs::remove_file(path)?;
        
        Ok(())
    }
}
