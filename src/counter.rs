use crate::config::CountShape;
use crate::preprocess::PreprocessedData;

pub struct CountResult {
    pub block_x: i32,
    pub block_z: i32,
    pub world_x: i32,
    pub world_z: i32,
    pub slime_count: usize,
    pub distance_sq: i64,
}

pub fn count_slime_chunks(
    prep: &PreprocessedData,
    grid: &[u8],
    shape: CountShape,
    size: i32,
    target: usize,
) -> Vec<CountResult> {
    let half_size = size / 2;
    let size_sq = (size as f64 / 2.0).powi(2) as i64;
    
    let mut results = Vec::new();
    
    for block_z in prep.min_block_z..=prep.max_block_z {
        for block_x in prep.min_block_x..=prep.max_block_x {
            let mut count = 0;
            
            for dz in -half_size..=half_size {
                for dx in -half_size..=half_size {
                    let check_z = block_z + dz;
                    let check_x = block_x + dx;
                    
                    let in_shape = match shape {
                        CountShape::Square => true,
                        CountShape::Circle => {
                            let dist_sq = (dx as i64).pow(2) + (dz as i64).pow(2);
                            dist_sq <= size_sq
                        }
                    };
                    
                    if !in_shape {
                        continue;
                    }
                    
                    if check_x < prep.min_block_x || check_x > prep.max_block_x ||
                       check_z < prep.min_block_z || check_z > prep.max_block_z {
                        continue;
                    }
                    
                    let x_idx = (check_x - prep.min_block_x) as usize;
                    let z_idx = (check_z - prep.min_block_z) as usize;
                    
                    if grid[z_idx * prep.x_count + x_idx] == 1 {
                        count += 1;
                    }
                }
            }
            
            if count > 0 {
                let dx = (block_x - prep.center_block_x) as i64;
                let dz = (block_z - prep.center_block_z) as i64;
                let distance_sq = dx * dx + dz * dz;
                
                results.push(CountResult {
                    block_x,
                    block_z,
                    world_x: block_x * 16,
                    world_z: block_z * 16,
                    slime_count: count,
                    distance_sq,
                });
            }
        }
    }
    
    results.sort_by(|a, b| {
        b.slime_count.cmp(&a.slime_count)
            .then_with(|| a.distance_sq.cmp(&b.distance_sq))
    });
    
    results.truncate(target);
    
    results
}
