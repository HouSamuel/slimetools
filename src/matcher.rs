use crate::config::Config;
use crate::preprocess::PreprocessedData;

pub struct MatchResult {
    pub block_x: i32,
    pub block_z: i32,
    pub world_x: i32,
    pub world_z: i32,
    pub distance_sq: i64,
}

pub fn find_matches(config: &Config, prep: &PreprocessedData, grid: &[u8]) -> Vec<MatchResult> {
    let pattern = match &prep.pattern {
        Some(p) => p,
        None => return Vec::new(),
    };
    
    let pattern_w = pattern.width;
    let pattern_h = pattern.height;
    
    if pattern_w == 0 || pattern_h == 0 {
        return Vec::new();
    }
    
    let max_x_idx = prep.x_count - pattern_w;
    let max_z_idx = prep.z_count - pattern_h;
    
    let mut matches = Vec::new();
    
    for z_idx in 0..=max_z_idx {
        for x_idx in 0..=max_x_idx {
            let mut match_found = true;
            
            for dz in 0..pattern_h {
                for dx in 0..pattern_w {
                    let gx = x_idx + dx;
                    let gz = z_idx + dz;
                    let grid_val = grid[gz * prep.x_count + gx];
                    let pattern_val = pattern.data[dz * pattern_w + dx];
                    
                    if pattern_val != 2 && grid_val != pattern_val {
                        match_found = false;
                        break;
                    }
                }
                if !match_found {
                    break;
                }
            }
            
            if match_found {
                let block_x = prep.min_block_x + x_idx as i32;
                let block_z = prep.min_block_z + z_idx as i32;
                
                let dx = (block_x - prep.center_block_x) as i64;
                let dz = (block_z - prep.center_block_z) as i64;
                let distance_sq = dx * dx + dz * dz;
                
                matches.push(MatchResult {
                    block_x,
                    block_z,
                    world_x: block_x * 16,
                    world_z: block_z * 16,
                    distance_sq,
                });
                
                if config.match_target > 0 && matches.len() >= config.match_target {
                    matches.sort_by(|a, b| a.distance_sq.cmp(&b.distance_sq));
                    return matches;
                }
            }
        }
    }
    
    matches.sort_by(|a, b| a.distance_sq.cmp(&b.distance_sq));
    
    if config.match_target > 0 {
        matches.truncate(config.match_target);
    }
    
    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Mode, TerminalMode};

    #[test]
    fn test_find_matches_empty_pattern() {
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
            output_map: false,
            output_match: false,
            output_count: false,
            output_log: true,
            terminal_mode: TerminalMode::Full,
        };
        
        let prep = crate::preprocess::preprocess(&config).unwrap();
        let grid = crate::grid::generate_grid(&config, &prep);
        let matches = find_matches(&config, &prep, &grid);
        
        assert!(matches.is_empty());
    }

    #[test]
    fn test_find_matches_pattern_any() {
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
            terminal_mode: TerminalMode::Full,
        };
        
        let prep = crate::preprocess::preprocess(&config).unwrap();
        let grid = crate::grid::generate_grid(&config, &prep);
        let matches = find_matches(&config, &prep, &grid);
        
        assert_eq!(matches.len(), (prep.x_count - 2 + 1) * (prep.z_count - 2 + 1));
    }

    #[test]
    fn test_find_matches_target_limit() {
        let config = Config {
            world_seed: 20260627,
            center_block_x: 0,
            center_block_z: 0,
            radius: 100,
            mode: Mode::Match,
            memory_limit_gib: None,
            pattern: Some(&[&[2, 2], &[2, 2]]),
            match_target: 5,
            count_shape: crate::config::CountShape::Square,
            count_size: 5,
            count_target: 10,
            output_map: false,
            output_match: true,
            output_count: false,
            output_log: true,
            terminal_mode: TerminalMode::Full,
        };
        
        let prep = crate::preprocess::preprocess(&config).unwrap();
        let grid = crate::grid::generate_grid(&config, &prep);
        let matches = find_matches(&config, &prep, &grid);
        
        assert_eq!(matches.len(), 5);
    }

    #[test]
    fn test_find_matches_boundary() {
        let config = Config {
            world_seed: 20260627,
            center_block_x: 0,
            center_block_z: 0,
            radius: 2,
            mode: Mode::Match,
            memory_limit_gib: None,
            pattern: Some(&[&[1, 1], &[1, 1]]),
            match_target: 0,
            count_shape: crate::config::CountShape::Square,
            count_size: 5,
            count_target: 10,
            output_map: false,
            output_match: true,
            output_count: false,
            output_log: true,
            terminal_mode: TerminalMode::Full,
        };
        
        let prep = crate::preprocess::preprocess(&config).unwrap();
        let grid = crate::grid::generate_grid(&config, &prep);
        let matches = find_matches(&config, &prep, &grid);
        
        for m in matches {
            assert!(m.block_x >= prep.min_block_x);
            assert!(m.block_x <= prep.max_block_x - 1);
            assert!(m.block_z >= prep.min_block_z);
            assert!(m.block_z <= prep.max_block_z - 1);
        }
    }
}