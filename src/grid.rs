use rayon::prelude::*;
use crate::config::Config;
use crate::preprocess::PreprocessedData;
use crate::slime_chunk::is_slime_chunk_fast;

const PARALLEL_THRESHOLD: usize = 10000;

pub fn generate_grid(config: &Config, prep: &PreprocessedData) -> Vec<u8> {
    let seed = config.world_seed as i64;
    let x_count = prep.x_count;
    
    let mut grid = vec![0u8; prep.total_blocks];
    
    if prep.total_blocks < PARALLEL_THRESHOLD {
        for z_idx in 0..prep.z_count {
            let fz = unsafe { *prep.fz_arr.get_unchecked(z_idx) };
            let row_start = z_idx * x_count;
            
            for x_idx in 0..x_count {
                let fx = unsafe { *prep.fx_arr.get_unchecked(x_idx) };
                unsafe { *grid.get_unchecked_mut(row_start + x_idx) = is_slime_chunk_fast(fx, fz, seed) as u8 };
            }
        }
    } else {
        grid.par_chunks_mut(x_count).enumerate().for_each(|(z_idx, row)| {
            let fz = unsafe { *prep.fz_arr.get_unchecked(z_idx) };
            
            for (x_idx, cell) in row.iter_mut().enumerate() {
                let fx = unsafe { *prep.fx_arr.get_unchecked(x_idx) };
                *cell = is_slime_chunk_fast(fx, fz, seed) as u8;
            }
        });
    }
    
    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Mode};

    #[test]
    fn test_generate_grid_size() {
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
            terminal_output: true,
        };
        
        let prep = crate::preprocess::preprocess(&config).unwrap();
        let grid = generate_grid(&config, &prep);
        
        assert_eq!(grid.len(), prep.total_blocks);
    }

    #[test]
    fn test_generate_grid_values() {
        let config = Config {
            world_seed: 20260627,
            center_block_x: 0,
            center_block_z: 0,
            radius: 5,
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
            terminal_output: true,
        };
        
        let prep = crate::preprocess::preprocess(&config).unwrap();
        let grid = generate_grid(&config, &prep);
        
        for &val in &grid {
            assert!(val == 0 || val == 1);
        }
    }

    #[test]
    fn test_generate_grid_consistency() {
        let config = Config {
            world_seed: 20260627,
            center_block_x: 0,
            center_block_z: 0,
            radius: 10,
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
            terminal_output: true,
        };
        
        let prep = crate::preprocess::preprocess(&config).unwrap();
        let grid1 = generate_grid(&config, &prep);
        let grid2 = generate_grid(&config, &prep);
        
        assert_eq!(grid1, grid2);
    }
}