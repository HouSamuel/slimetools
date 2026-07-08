use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
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

pub fn generate_grid_with_progress(
    config: &Config,
    prep: &PreprocessedData,
    progress_counter: &AtomicUsize,
) -> Vec<u8> {
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
            progress_counter.fetch_add(1, Ordering::Relaxed);
        }
    } else {
        grid.par_chunks_mut(x_count).enumerate().for_each(|(z_idx, row)| {
            let fz = unsafe { *prep.fz_arr.get_unchecked(z_idx) };
            
            for (x_idx, cell) in row.iter_mut().enumerate() {
                let fx = unsafe { *prep.fx_arr.get_unchecked(x_idx) };
                *cell = is_slime_chunk_fast(fx, fz, seed) as u8;
            }
            progress_counter.fetch_add(1, Ordering::Relaxed);
        });
    }
    
    grid
}

pub fn generate_grid_chunk(
    config: &Config,
    prep: &PreprocessedData,
    z_start: usize,
    z_end: usize,
) -> Vec<u8> {
    let seed = config.world_seed as i64;
    let x_count = prep.x_count;
    let chunk_rows = z_end - z_start;
    
    let mut chunk = vec![0u8; chunk_rows * x_count];
    
    for (chunk_z_idx, z_idx) in (z_start..z_end).enumerate() {
        let fz = unsafe { *prep.fz_arr.get_unchecked(z_idx) };
        let row_start = chunk_z_idx * x_count;
        
        for x_idx in 0..x_count {
            let fx = unsafe { *prep.fx_arr.get_unchecked(x_idx) };
            unsafe { *chunk.get_unchecked_mut(row_start + x_idx) = is_slime_chunk_fast(fx, fz, seed) as u8 };
        }
    }
    
    chunk
}

pub fn generate_grid_chunk_with_progress(
    config: &Config,
    prep: &PreprocessedData,
    z_start: usize,
    z_end: usize,
    progress_counter: &AtomicUsize,
) -> Vec<u8> {
    let seed = config.world_seed as i64;
    let x_count = prep.x_count;
    let chunk_rows = z_end - z_start;
    
    let mut chunk = vec![0u8; chunk_rows * x_count];
    
    if chunk_rows < PARALLEL_THRESHOLD {
        for (chunk_z_idx, z_idx) in (z_start..z_end).enumerate() {
            let fz = unsafe { *prep.fz_arr.get_unchecked(z_idx) };
            let row_start = chunk_z_idx * x_count;
            
            for x_idx in 0..x_count {
                let fx = unsafe { *prep.fx_arr.get_unchecked(x_idx) };
                unsafe { *chunk.get_unchecked_mut(row_start + x_idx) = is_slime_chunk_fast(fx, fz, seed) as u8 };
            }
            progress_counter.fetch_add(1, Ordering::Relaxed);
        }
    } else {
        let fz_arr = &prep.fz_arr;
        let fx_arr = &prep.fx_arr;
        
        chunk.par_chunks_mut(x_count).enumerate().for_each(|(chunk_z_idx, row)| {
            let z_idx = z_start + chunk_z_idx;
            let fz = unsafe { *fz_arr.get_unchecked(z_idx) };
            
            for (x_idx, cell) in row.iter_mut().enumerate() {
                let fx = unsafe { *fx_arr.get_unchecked(x_idx) };
                *cell = is_slime_chunk_fast(fx, fz, seed) as u8;
            }
            progress_counter.fetch_add(1, Ordering::Relaxed);
        });
    }
    
    chunk
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Mode, TerminalMode};

    #[test]
    fn test_generate_grid_size() {
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
        
        let prep = crate::preprocess::preprocess(&config).unwrap();
        let grid1 = generate_grid(&config, &prep);
        let grid2 = generate_grid(&config, &prep);
        
        assert_eq!(grid1, grid2);
    }
}