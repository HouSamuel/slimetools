use rayon::prelude::*;
use crate::config::Config;
use crate::preprocess::PreprocessedData;
use crate::slime_chunk::{FastRandom, is_slime_chunk_fast};

pub fn generate_grid(config: &Config, prep: &PreprocessedData) -> Vec<u8> {
    let seed = config.world_seed as i64;
    
    (0..prep.z_count)
        .into_par_iter()
        .flat_map(|z_idx| {
            let fz = prep.fz_arr[z_idx];
            let mut rng = FastRandom::new();
            
            (0..prep.x_count)
                .map(move |x_idx| {
                    let fx = prep.fx_arr[x_idx];
                    if is_slime_chunk_fast(&mut rng, fx, fz, seed) {
                        1u8
                    } else {
                        0u8
                    }
                })
                .collect::<Vec<u8>>()
        })
        .collect()
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
