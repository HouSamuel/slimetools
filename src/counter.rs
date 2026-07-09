use crate::config::CountShape;
use crate::preprocess::PreprocessedData;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::atomic::AtomicUsize;

pub struct CountResult {
    pub block_x: i32,
    pub block_z: i32,
    pub world_x: i32,
    pub world_z: i32,
    pub slime_count: usize,
    pub distance_sq: i64,
}

impl PartialEq for CountResult {
    fn eq(&self, other: &Self) -> bool {
        self.slime_count == other.slime_count && self.distance_sq == other.distance_sq
    }
}

impl Eq for CountResult {}

impl PartialOrd for CountResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CountResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.slime_count.cmp(&other.slime_count)
            .then_with(|| other.distance_sq.cmp(&self.distance_sq))
    }
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
    
    let mut heap: BinaryHeap<Reverse<CountResult>> = BinaryHeap::new();
    
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
                
                let result = CountResult {
                    block_x,
                    block_z,
                    world_x: block_x * 16,
                    world_z: block_z * 16,
                    slime_count: count,
                    distance_sq,
                };
                
                if heap.len() < target {
                    heap.push(Reverse(result));
                } else if let Some(Reverse(top)) = heap.peek() {
                    if result > *top {
                        heap.pop();
                        heap.push(Reverse(result));
                    }
                }
            }
        }
    }
    
    let mut results: Vec<CountResult> = heap.into_iter().map(|Reverse(r)| r).collect();
    results.sort_by(|a, b| {
        b.slime_count.cmp(&a.slime_count)
            .then_with(|| a.distance_sq.cmp(&b.distance_sq))
    });
    
    results
}

pub fn count_slime_chunks_chunk(
    prep: &PreprocessedData,
    chunk: &[u8],
    chunk_z_start: usize,
    chunk_z_end: usize,
    shape: CountShape,
    size: i32,
    target: usize,
) -> Vec<CountResult> {
    let half_size = size / 2;
    let size_sq = (size as f64 / 2.0).powi(2) as i64;
    
    let mut heap: BinaryHeap<Reverse<CountResult>> = BinaryHeap::new();
    
    let chunk_start_block_z = prep.min_block_z + chunk_z_start as i32;
    let chunk_end_block_z = prep.min_block_z + chunk_z_end as i32 - 1;
    
    let valid_start_z = std::cmp::max(chunk_start_block_z + half_size, prep.min_block_z);
    let valid_end_z = std::cmp::min(chunk_end_block_z - half_size, prep.max_block_z);
    
    for block_z in valid_start_z..=valid_end_z {
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
                    
                    if z_idx < chunk_z_start || z_idx >= chunk_z_end {
                        continue;
                    }
                    
                    let chunk_z_idx = z_idx - chunk_z_start;
                    
                    if chunk[chunk_z_idx * prep.x_count + x_idx] == 1 {
                        count += 1;
                    }
                }
            }
            
            if count > 0 {
                let dx = (block_x - prep.center_block_x) as i64;
                let dz = (block_z - prep.center_block_z) as i64;
                let distance_sq = dx * dx + dz * dz;
                
                let result = CountResult {
                    block_x,
                    block_z,
                    world_x: block_x * 16,
                    world_z: block_z * 16,
                    slime_count: count,
                    distance_sq,
                };
                
                if heap.len() < target {
                    heap.push(Reverse(result));
                } else if let Some(Reverse(top)) = heap.peek() {
                    if result > *top {
                        heap.pop();
                        heap.push(Reverse(result));
                    }
                }
            }
        }
    }
    
    let mut results: Vec<CountResult> = heap.into_iter().map(|Reverse(r)| r).collect();
    results.sort_by(|a, b| {
        b.slime_count.cmp(&a.slime_count)
            .then_with(|| a.distance_sq.cmp(&b.distance_sq))
    });
    
    results
}

pub fn count_slime_chunks_chunk_with_progress(
    prep: &PreprocessedData,
    chunk: &[u8],
    chunk_z_start: usize,
    chunk_z_end: usize,
    shape: CountShape,
    size: i32,
    target: usize,
    progress: &AtomicUsize,
) -> Vec<CountResult> {
    let half_size = size / 2;
    let size_sq = (size as f64 / 2.0).powi(2) as i64;
    
    let mut heap: BinaryHeap<Reverse<CountResult>> = BinaryHeap::new();
    
    let chunk_start_block_z = prep.min_block_z + chunk_z_start as i32;
    let chunk_end_block_z = prep.min_block_z + chunk_z_end as i32 - 1;
    
    let valid_start_z = std::cmp::max(chunk_start_block_z + half_size, prep.min_block_z);
    let valid_end_z = std::cmp::min(chunk_end_block_z - half_size, prep.max_block_z);
    
    let total_iterations = ((valid_end_z - valid_start_z + 1) as usize) * ((prep.max_block_x - prep.min_block_x + 1) as usize);
    let mut iteration = 0;
    
    for block_z in valid_start_z..=valid_end_z {
        for block_x in prep.min_block_x..=prep.max_block_x {
            iteration += 1;
            if iteration % 100 == 0 || iteration == total_iterations {
                progress.store(iteration, std::sync::atomic::Ordering::Relaxed);
            }
            
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
                    
                    if z_idx < chunk_z_start || z_idx >= chunk_z_end {
                        continue;
                    }
                    
                    let chunk_z_idx = z_idx - chunk_z_start;
                    
                    if chunk[chunk_z_idx * prep.x_count + x_idx] == 1 {
                        count += 1;
                    }
                }
            }
            
            if count > 0 {
                let dx = (block_x - prep.center_block_x) as i64;
                let dz = (block_z - prep.center_block_z) as i64;
                let distance_sq = dx * dx + dz * dz;
                
                let result = CountResult {
                    block_x,
                    block_z,
                    world_x: block_x * 16,
                    world_z: block_z * 16,
                    slime_count: count,
                    distance_sq,
                };
                
                if heap.len() < target {
                    heap.push(Reverse(result));
                } else if let Some(Reverse(top)) = heap.peek() {
                    if result > *top {
                        heap.pop();
                        heap.push(Reverse(result));
                    }
                }
            }
        }
    }
    
    progress.store(total_iterations, std::sync::atomic::Ordering::Relaxed);
    
    let mut results: Vec<CountResult> = heap.into_iter().map(|Reverse(r)| r).collect();
    results.sort_by(|a, b| {
        b.slime_count.cmp(&a.slime_count)
            .then_with(|| a.distance_sq.cmp(&b.distance_sq))
    });
    
    results
}
