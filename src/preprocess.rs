use crate::config::{Config, Mode};
use crate::slime_chunk::{compute_fx, compute_fz};

#[derive(Debug, Clone, Copy)]
pub struct Pattern {
    pub data: &'static [u8],
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone)]
pub struct PreprocessedData {
    pub center_block_x: i32,
    pub center_block_z: i32,
    pub center_world_x: i32,
    pub center_world_z: i32,
    
    pub min_block_x: i32,
    pub max_block_x: i32,
    pub min_block_z: i32,
    pub max_block_z: i32,
    
    pub top_left_block: (i32, i32),
    pub bottom_right_block: (i32, i32),
    pub top_left_world: (i32, i32),
    pub bottom_right_world: (i32, i32),
    
    pub x_count: usize,
    pub z_count: usize,
    pub total_blocks: usize,
    
    pub fx_arr: Vec<i64>,
    pub fz_arr: Vec<i64>,
    
    pub z_headers: Vec<Vec<u8>>,
    pub x_headers: Vec<Vec<u8>>,
    pub z_col_width: usize,
    
    pub one_cell: Vec<u8>,
    pub zero_cell: Vec<u8>,
    
    pub chunk_count: usize,
    pub chunk_size: usize,
    
    pub pattern: Option<Pattern>,
}

#[inline(always)]
fn num_width(mut n: i32) -> usize {
    if n == 0 {
        return 1;
    }
    let mut w = if n < 0 { 1 } else { 0 };
    n = n.abs();
    while n > 0 {
        w += 1;
        n /= 10;
    }
    w
}

fn flatten_pattern(pattern_rows: &[&[u8]]) -> Pattern {
    let height = pattern_rows.len();
    let width = if height == 0 { 0 } else { pattern_rows[0].len() };
    let size = width * height;
    
    let mut arr = Vec::with_capacity(size);
    for &row in pattern_rows {
        arr.extend_from_slice(row);
    }
    
    Pattern {
        data: Box::leak(arr.into_boxed_slice()),
        width,
        height,
    }
}

fn generate_challenge_code() -> u32 {
    let now = chrono::Local::now();
    let timestamp = now.timestamp();
    let minutes_since_epoch = timestamp / 60;
    let ten_minute_block = minutes_since_epoch / 10;
    ten_minute_block as u32
}

fn validate(config: &Config) -> Result<(), String> {
    if config.radius < 0 {
        return Err("半径不能为负数".to_string());
    }
    
    if config.center_block_x.abs() > 10000000 || config.center_block_z.abs() > 10000000 {
        return Err("区块坐标超出合理范围".to_string());
    }
    
    match config.mode {
        Mode::Match => {
            if config.pattern.is_none() {
                return Err("匹配模式需要设置图案".to_string());
            }
            let pattern_rows = config.pattern.unwrap();
            if pattern_rows.is_empty() {
                return Err("图案行数不能为0".to_string());
            }
            let width = pattern_rows[0].len();
            if width == 0 {
                return Err("图案列数不能为0".to_string());
            }
            for &row in pattern_rows {
                if row.len() != width {
                    return Err("图案每行长度必须一致".to_string());
                }
                for &val in row {
                    if val > 2 {
                        return Err("图案值只能是0、1、2".to_string());
                    }
                }
            }
        }
        Mode::Count => {
            if config.count_size <= 0 {
                return Err("计数区域尺寸必须大于0".to_string());
            }
            if config.count_target == 0 {
                return Err("计数目标数量必须大于0".to_string());
            }
        }
        Mode::Check => {}
    }
    
    if config.memory_limit_gib < 0.0 {
        return Err("内存限制不能为负数".to_string());
    }
    
    if config.progress_update_interval <= 0.0 || config.progress_update_interval > 100.0 {
        return Err("进度更新分度值必须在 0.01-100.0 之间".to_string());
    }
    
    Ok(())
}

fn estimate_map_file_size(_config: &Config, x_count: usize, z_count: usize) -> f64 {
    let max_coord = (x_count / 2) as i32;
    let num_digits = if max_coord == 0 {
        1
    } else {
        ((max_coord as f64).log10().floor() + 1.0) as usize + 1
    };
    let cell_width = num_digits + 1;
    let z_col_width = num_digits + 1;
    let header_lines = 10;
    let line_ending = 2;
    
    let line_size = (z_col_width + x_count * cell_width + line_ending) as f64;
    let total_size_bytes = (z_count + header_lines) as f64 * line_size;
    total_size_bytes / (1024.0 * 1024.0 * 1024.0)
}

fn should_require_challenge(config: &Config, x_count: usize, z_count: usize) -> bool {
    config.secure_mode && config.output_map && estimate_map_file_size(config, x_count, z_count) > 5.0
}

pub fn preprocess(config: &Config) -> Result<PreprocessedData, String> {
    validate(config)?;
    
    let min_block_x = config.center_block_x - config.radius;
    let max_block_x = config.center_block_x + config.radius;
    let min_block_z = config.center_block_z - config.radius;
    let max_block_z = config.center_block_z + config.radius;
    
    let x_count = (max_block_x - min_block_x + 1) as usize;
    let z_count = (max_block_z - min_block_z + 1) as usize;
    let total_blocks = x_count * z_count;
    
    if should_require_challenge(config, x_count, z_count) {
        let correct_code = generate_challenge_code();
        match config.challenge_code {
            Some(input_code) => {
                if input_code != correct_code {
                    return Err(format!("挑战码错误！正确挑战码: 0x{:08x}", correct_code));
                }
            }
            None => {
                return Err(format!("安全模式已启用，预估map文件大于5GiB，请输入挑战码！当前挑战码: 0x{:08x}", correct_code));
            }
        }
    }
    
    let center_world_x = config.center_block_x * 16;
    let center_world_z = config.center_block_z * 16;
    
    let top_left_block = (min_block_x, max_block_z);
    let bottom_right_block = (max_block_x, min_block_z);
    let top_left_world = (min_block_x * 16, max_block_z * 16 + 15);
    let bottom_right_world = (max_block_x * 16 + 15, min_block_z * 16);
    
    let fx_arr: Vec<i64> = (min_block_x..=max_block_x)
        .map(|x| compute_fx(x))
        .collect();
    
    let fz_arr: Vec<i64> = (min_block_z..=max_block_z)
        .map(|z| compute_fz(z))
        .collect();
    
    let z_col_width = std::cmp::max(
        num_width(min_block_z),
        num_width(max_block_z),
    );
    
    let z_headers: Vec<Vec<u8>> = (min_block_z..=max_block_z)
        .rev()
        .map(|z| {
            let s = format!("{:>width$}|", z, width = z_col_width);
            s.into_bytes()
        })
        .collect();
    
    let x_header_width = std::cmp::max(
        num_width(min_block_x),
        num_width(max_block_x),
    );
    let cell_width = x_header_width + 1;
    
    let x_headers: Vec<Vec<u8>> = (min_block_x..=max_block_x)
        .map(|x| {
            format!("{:>width$} ", x, width = x_header_width).into_bytes()
        })
        .collect();
    
    let mut zero_cell = vec![b' '; cell_width];
    zero_cell[0] = b'0';
    let mut one_cell = vec![b' '; cell_width];
    one_cell[0] = b'1';
    
    let memory_limit_bytes = if config.memory_limit_gib > 0.0 {
        Some((config.memory_limit_gib * 1024.0 * 1024.0 * 1024.0) as usize)
    } else {
        None
    };
    let (chunk_count, chunk_size) = calculate_chunking(total_blocks, memory_limit_bytes);
    
    let pattern = config.pattern.map(flatten_pattern);
    
    Ok(PreprocessedData {
        center_block_x: config.center_block_x,
        center_block_z: config.center_block_z,
        center_world_x,
        center_world_z,
        min_block_x,
        max_block_x,
        min_block_z,
        max_block_z,
        top_left_block,
        bottom_right_block,
        top_left_world,
        bottom_right_world,
        x_count,
        z_count,
        total_blocks,
        fx_arr,
        fz_arr,
        z_headers,
        x_headers,
        z_col_width,
        one_cell,
        zero_cell,
        chunk_count,
        chunk_size,
        pattern,
    })
}

fn calculate_chunking(total_blocks: usize, memory_limit_bytes: Option<usize>) -> (usize, usize) {
    const GRID_BYTES_PER_BLOCK: usize = 1;
    const SAFETY_MARGIN: f64 = 0.8;
    
    let grid_memory = total_blocks * GRID_BYTES_PER_BLOCK;
    
    if let Some(limit) = memory_limit_bytes {
        let available = (limit as f64 * SAFETY_MARGIN) as usize;
        if grid_memory <= available {
            return (1, total_blocks);
        }
        let chunk_size = available / GRID_BYTES_PER_BLOCK;
        let chunk_count = (total_blocks + chunk_size - 1) / chunk_size;
        (chunk_count, chunk_size)
    } else {
        (1, total_blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Mode, TerminalMode};

    #[test]
    fn test_num_width() {
        assert_eq!(num_width(0), 1);
        assert_eq!(num_width(5), 1);
        assert_eq!(num_width(10), 2);
        assert_eq!(num_width(99), 2);
        assert_eq!(num_width(100), 3);
        assert_eq!(num_width(-1), 2);
        assert_eq!(num_width(-10), 3);
    }

    #[test]
    fn test_preprocess_basic() {
        let config = Config {
            world_seed: 20260627,
            center_block_x: 0,
            center_block_z: 0,
            radius: 2,
            mode: Mode::Check,
            memory_limit_gib: 0.0,
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
            secure_mode: false,
            challenge_code: None,
            progress_update_interval: 1.0,
        };
        
        let prep = preprocess(&config).unwrap();
        
        assert_eq!(prep.min_block_x, -2);
        assert_eq!(prep.max_block_x, 2);
        assert_eq!(prep.min_block_z, -2);
        assert_eq!(prep.max_block_z, 2);
        assert_eq!(prep.x_count, 5);
        assert_eq!(prep.z_count, 5);
        assert_eq!(prep.total_blocks, 25);
        assert_eq!(prep.fx_arr.len(), 5);
        assert_eq!(prep.fz_arr.len(), 5);
    }

    #[test]
    fn test_preprocess_offset_center() {
        let config = Config {
            world_seed: 20260627,
            center_block_x: 10,
            center_block_z: 20,
            radius: 1,
            mode: Mode::Check,
            memory_limit_gib: 0.0,
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
            secure_mode: false,
            challenge_code: None,
            progress_update_interval: 1.0,
        };
        
        let prep = preprocess(&config).unwrap();
        
        assert_eq!(prep.min_block_x, 9);
        assert_eq!(prep.max_block_x, 11);
        assert_eq!(prep.min_block_z, 19);
        assert_eq!(prep.max_block_z, 21);
        assert_eq!(prep.center_world_x, 160);
        assert_eq!(prep.center_world_z, 320);
    }

    #[test]
    fn test_preprocess_world_coords() {
        let config = Config {
            world_seed: 20260627,
            center_block_x: 2,
            center_block_z: 3,
            radius: 1,
            mode: Mode::Check,
            memory_limit_gib: 0.0,
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
            secure_mode: false,
            challenge_code: None,
            progress_update_interval: 1.0,
        };
        
        let prep = preprocess(&config).unwrap();
        
        assert_eq!(prep.top_left_world.0, 16);
        assert_eq!(prep.bottom_right_world.0, 63);
        assert_eq!(prep.bottom_right_world.1, 32);
        assert_eq!(prep.top_left_world.1, 79);
    }

    #[test]
    fn test_flatten_pattern() {
        let pattern_rows: &[&[u8]] = &[&[1u8, 2u8], &[3u8, 4u8]];
        let pattern = flatten_pattern(pattern_rows);
        
        assert_eq!(pattern.width, 2);
        assert_eq!(pattern.height, 2);
        assert_eq!(pattern.data, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_validate_match_pattern() {
        let config = Config {
            world_seed: 20260627,
            center_block_x: 0,
            center_block_z: 0,
            radius: 10,
            mode: Mode::Match,
            memory_limit_gib: 0.0,
            pattern: Some(&[&[1, 1], &[1, 1]]),
            match_target: 0,
            count_shape: crate::config::CountShape::Square,
            count_size: 5,
            count_target: 10,
            output_map: true,
            output_match: true,
            output_count: false,
            output_log: true,
            terminal_mode: TerminalMode::Full,
            secure_mode: false,
            challenge_code: None,
            progress_update_interval: 1.0,
        };
        
        assert!(validate(&config).is_ok());
    }

    #[test]
    fn test_validate_pattern_invalid_value() {
        let config = Config {
            world_seed: 20260627,
            center_block_x: 0,
            center_block_z: 0,
            radius: 10,
            mode: Mode::Match,
            memory_limit_gib: 0.0,
            pattern: Some(&[&[1, 3], &[1, 1]]),
            match_target: 0,
            count_shape: crate::config::CountShape::Square,
            count_size: 5,
            count_target: 10,
            output_map: true,
            output_match: true,
            output_count: false,
            output_log: true,
            terminal_mode: TerminalMode::Full,
            secure_mode: false,
            challenge_code: None,
            progress_update_interval: 1.0,
        };
        
        assert!(validate(&config).is_err());
    }
}