use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

mod config;
mod slime_chunk;
mod preprocess;
mod grid;
mod matcher;
mod counter;
mod grid_output;
mod match_output;
mod count_output;
mod info;
mod terminal;
mod log;

use config::CONFIG;
use terminal::*;

fn main() -> std::io::Result<()> {
    let total_start = Instant::now();
    
    let config = &CONFIG;
    
    let prep_start = Instant::now();
    let prep = preprocess::preprocess(config).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let prep_time = prep_start.elapsed();
    
    print_info(config, &prep);
    
    let mut log_writer = log::LogWriter::new(config).unwrap_or_else(|| {
        log::LogWriter {
            inner: Arc::new(std::sync::Mutex::new(None)),
        }
    });
    log_writer.write_info(config, &prep);
    
    let mut total_slime_count = 0usize;
    let grid_start = Instant::now();
    
    match config.mode {
        config::Mode::Check => {
            process_check_mode(config, &prep, &mut total_slime_count, &mut log_writer)?;
        }
        
        config::Mode::Match => {
            process_match_mode(config, &prep, &mut total_slime_count, &mut log_writer)?;
        }
        
        config::Mode::Count => {
            process_count_mode(config, &prep, &mut total_slime_count, &mut log_writer)?;
        }
    }
    
    let grid_time = grid_start.elapsed();
    let total_time = total_start.elapsed();
    
    print_stats(config, prep_time, grid_time, grid_time, grid_time, total_time, total_slime_count, prep.total_blocks);
    log_writer.write_stats(prep_time, grid_time, grid_time, grid_time, total_time, total_slime_count, prep.total_blocks);
    
    Ok(())
}

fn process_check_mode(
    config: &config::Config,
    prep: &preprocess::PreprocessedData,
    total_slime_count: &mut usize,
    log_writer: &mut log::LogWriter,
) -> std::io::Result<()> {
    let chunk_rows = if prep.chunk_count > 1 {
        prep.chunk_size / prep.x_count
    } else {
        prep.z_count
    };
    
    let progress_display = Arc::new(Mutex::new(ProgressDisplay::new(prep.chunk_count, chunk_rows, config.progress_update_interval)));
    progress_display.lock().unwrap().print_header(prep);
    log_writer.write_progress_header(config, prep);
    
    if prep.chunk_count > 1 {
        let mut all_slime_count = 0;
        let chunk_count_val = prep.chunk_count;
        
        for chunk_idx in 0..prep.chunk_count {
            let z_start = chunk_idx * (prep.chunk_size / prep.x_count);
            let z_end = std::cmp::min(z_start + (prep.chunk_size / prep.x_count), prep.z_count);
            let current_chunk_rows = z_end - z_start;
            
            progress_display.lock().unwrap().start_chunk(chunk_idx);
            log_writer.write_chunk_start(chunk_idx, chunk_count_val);
            
            let progress_counter = Arc::new(AtomicUsize::new(0));
            let counter_clone = Arc::clone(&progress_counter);
            let mut log_clone = log_writer.clone();
            let display_clone = Arc::clone(&progress_display);
            let chunk_idx_val = chunk_idx;
            
            let start_time = Instant::now();
            let handle = thread::spawn(move || {
                while counter_clone.load(Ordering::Relaxed) < current_chunk_rows {
                    let count = counter_clone.load(Ordering::Relaxed);
                    let percent = (count as f64 / current_chunk_rows as f64) * 100.0;
                    let elapsed = start_time.elapsed();
                    
                    display_clone.lock().unwrap().update_grid(percent, elapsed);
                    log_clone.write_chunk_update(chunk_idx_val, chunk_count_val, percent, elapsed);
                    
                    thread::sleep(Duration::from_millis(500));
                }
            });
            
            let chunk = grid::generate_grid_chunk_with_progress(config, prep, z_start, z_end, &progress_counter);
            
            handle.join().unwrap();
            
            let grid_elapsed = start_time.elapsed();
            progress_display.lock().unwrap().finish_grid(grid_elapsed);
            log_writer.write_chunk_finish(chunk_idx, chunk_count_val, grid_elapsed);
            
            all_slime_count += chunk.iter().filter(|&&x| x == 1).count();
            
            progress_display.lock().unwrap().finish_process(Duration::from_secs(0));
            
            if config.output_map {
                let path = grid_output::output_grid_chunk(config, prep, &chunk, z_start, z_end)?;
                print_grid_file_written(config, &path);
            }
        }
        
        progress_display.lock().unwrap().finish_all();
        log_writer.write_progress_finish();
        *total_slime_count = all_slime_count;
    } else {
        let progress_counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&progress_counter);
        let mut log_clone = log_writer.clone();
        let display_clone = Arc::clone(&progress_display);
        let z_count_val = prep.z_count;
        
        progress_display.lock().unwrap().start_chunk(0);
        log_writer.write_chunk_start(0, 1);
        
        let start_time = Instant::now();
        let handle = thread::spawn(move || {
            while counter_clone.load(Ordering::Relaxed) < z_count_val {
                let count = counter_clone.load(Ordering::Relaxed);
                let percent = (count as f64 / z_count_val as f64) * 100.0;
                let elapsed = start_time.elapsed();
                
                display_clone.lock().unwrap().update_grid(percent, elapsed);
                log_clone.write_chunk_update(0, 1, percent, elapsed);
                
                thread::sleep(Duration::from_millis(500));
            }
        });
        
        let grid = grid::generate_grid_with_progress(config, prep, &progress_counter);
        
        handle.join().unwrap();
        
        let grid_elapsed = start_time.elapsed();
        progress_display.lock().unwrap().finish_grid(grid_elapsed);
        log_writer.write_chunk_finish(0, 1, grid_elapsed);
        
        *total_slime_count = grid.iter().filter(|&&x| x == 1).count();
        
        progress_display.lock().unwrap().finish_process(Duration::from_secs(0));
        progress_display.lock().unwrap().finish_all();
        log_writer.write_progress_finish();
        
        if config.output_map {
            let path = grid_output::output_grid_file(config, prep, &grid)?;
            print_grid_file_written(config, &path);
        }
    }
    
    Ok(())
}

fn process_match_mode(
    config: &config::Config,
    prep: &preprocess::PreprocessedData,
    total_slime_count: &mut usize,
    log_writer: &mut log::LogWriter,
) -> std::io::Result<()> {
    let chunk_rows = if prep.chunk_count > 1 {
        prep.chunk_size / prep.x_count
    } else {
        prep.z_count
    };
    
    let progress_display = Arc::new(Mutex::new(ProgressDisplay::new(prep.chunk_count, chunk_rows, config.progress_update_interval)));
    progress_display.lock().unwrap().print_header(prep);
    log_writer.write_progress_header(config, prep);
    
    let mut all_matches = Vec::new();
    let mut all_slime_count = 0;
    
    if prep.chunk_count > 1 {
        let chunk_count_val = prep.chunk_count;
        
        for chunk_idx in 0..prep.chunk_count {
            let z_start = chunk_idx * (prep.chunk_size / prep.x_count);
            let z_end = std::cmp::min(z_start + (prep.chunk_size / prep.x_count), prep.z_count);
            let current_chunk_rows = z_end - z_start;
            
            progress_display.lock().unwrap().start_chunk(chunk_idx);
            log_writer.write_chunk_start(chunk_idx, chunk_count_val);
            
            let progress_counter = Arc::new(AtomicUsize::new(0));
            let counter_clone = Arc::clone(&progress_counter);
            let mut log_clone = log_writer.clone();
            let display_clone = Arc::clone(&progress_display);
            let chunk_idx_val = chunk_idx;
            
            let start_time = Instant::now();
            let handle = thread::spawn(move || {
                while counter_clone.load(Ordering::Relaxed) < current_chunk_rows {
                    let count = counter_clone.load(Ordering::Relaxed);
                    let percent = (count as f64 / current_chunk_rows as f64) * 100.0;
                    let elapsed = start_time.elapsed();
                    
                    display_clone.lock().unwrap().update_grid(percent, elapsed);
                    log_clone.write_chunk_update(chunk_idx_val, chunk_count_val, percent, elapsed);
                    
                    thread::sleep(Duration::from_millis(500));
                }
            });
            
            let chunk = grid::generate_grid_chunk_with_progress(config, prep, z_start, z_end, &progress_counter);
            
            handle.join().unwrap();
            
            let grid_elapsed = start_time.elapsed();
            progress_display.lock().unwrap().finish_grid(grid_elapsed);
            log_writer.write_chunk_finish(chunk_idx, chunk_count_val, grid_elapsed);
            
            all_slime_count += chunk.iter().filter(|&&x| x == 1).count();
            
            let max_x_idx = prep.x_count - prep.pattern.as_ref().map(|p| p.width).unwrap_or(0);
            let max_z_idx = z_end - z_start - prep.pattern.as_ref().map(|p| p.height).unwrap_or(0);
            let process_total = (max_z_idx + 1) * (max_x_idx + 1);
            
            let process_progress = Arc::new(AtomicUsize::new(0));
            let process_counter_clone = Arc::clone(&process_progress);
            let process_display_clone = Arc::clone(&progress_display);
            let mut process_log_clone = log_writer.clone();
            let grid_elapsed_clone = grid_elapsed;
            
            let process_start = Instant::now();
            let process_handle = thread::spawn(move || {
                while process_counter_clone.load(Ordering::Relaxed) < process_total {
                    let count = process_counter_clone.load(Ordering::Relaxed);
                    let percent = (count as f64 / process_total as f64) * 100.0;
                    let elapsed = process_start.elapsed();
                    
                    process_display_clone.lock().unwrap().update_process(percent, elapsed);
                    process_log_clone.write_process_update(chunk_idx, chunk_count_val, grid_elapsed_clone, percent, elapsed);
                    
                    thread::sleep(Duration::from_millis(500));
                }
            });
            
            let chunk_matches = matcher::find_matches_chunk_with_progress(config, prep, &chunk, z_start, z_end, &process_progress);
            
            process_handle.join().unwrap();
            
            let process_elapsed = process_start.elapsed();
            progress_display.lock().unwrap().finish_process(process_elapsed);
            log_writer.write_process_finish(chunk_idx, chunk_count_val, grid_elapsed, process_elapsed);
            
            all_matches.extend(chunk_matches);
            
            if config.match_target > 0 && all_matches.len() >= config.match_target {
                progress_display.lock().unwrap().finish_all();
                log_writer.write_progress_finish();
                break;
            }
        }
        
        if all_matches.len() < config.match_target || config.match_target == 0 {
            progress_display.lock().unwrap().finish_all();
            log_writer.write_progress_finish();
        }
    } else {
        let progress_counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&progress_counter);
        let mut log_clone = log_writer.clone();
        let display_clone = Arc::clone(&progress_display);
        let z_count_val = prep.z_count;
        
        progress_display.lock().unwrap().start_chunk(0);
        log_writer.write_chunk_start(0, 1);
        
        let start_time = Instant::now();
        let handle = thread::spawn(move || {
            while counter_clone.load(Ordering::Relaxed) < z_count_val {
                let count = counter_clone.load(Ordering::Relaxed);
                let percent = (count as f64 / z_count_val as f64) * 100.0;
                let elapsed = start_time.elapsed();
                
                display_clone.lock().unwrap().update_grid(percent, elapsed);
                log_clone.write_chunk_update(0, 1, percent, elapsed);
                
                thread::sleep(Duration::from_millis(500));
            }
        });
        
        let grid = grid::generate_grid_with_progress(config, prep, &progress_counter);
        
        handle.join().unwrap();
        
        let grid_elapsed = start_time.elapsed();
        progress_display.lock().unwrap().finish_grid(grid_elapsed);
        log_writer.write_chunk_finish(0, 1, grid_elapsed);
        
        all_slime_count = grid.iter().filter(|&&x| x == 1).count();
        
        let process_start = Instant::now();
        all_matches = matcher::find_matches(config, prep, &grid);
        let process_elapsed = process_start.elapsed();
        
        progress_display.lock().unwrap().finish_process(process_elapsed);
        progress_display.lock().unwrap().finish_all();
        log_writer.write_progress_finish();
        
        if config.output_map {
            let path = grid_output::output_grid_file(config, prep, &grid)?;
            print_grid_file_written(config, &path);
        }
    }
    
    *total_slime_count = all_slime_count;
    
    all_matches.sort_by(|a, b| a.distance_sq.cmp(&b.distance_sq));
    
    if config.match_target > 0 {
        all_matches.truncate(config.match_target);
    }
    
    if config.output_match {
        let match_path = match_output::output_match_file(config, prep, &all_matches)?;
        print_match_file_written(config, &match_path);
        log_writer.write_match_file_written(&match_path);
    }
    
    print_match_summary(config, &all_matches);
    log_writer.write_match_summary(&all_matches);
    
    Ok(())
}

fn process_count_mode(
    config: &config::Config,
    prep: &preprocess::PreprocessedData,
    total_slime_count: &mut usize,
    log_writer: &mut log::LogWriter,
) -> std::io::Result<()> {
    let chunk_rows = if prep.chunk_count > 1 {
        prep.chunk_size / prep.x_count
    } else {
        prep.z_count
    };
    
    let progress_display = Arc::new(Mutex::new(ProgressDisplay::new(prep.chunk_count, chunk_rows, config.progress_update_interval)));
    progress_display.lock().unwrap().print_header(prep);
    log_writer.write_progress_header(config, prep);
    
    let mut all_results: BinaryHeap<Reverse<counter::CountResult>> = BinaryHeap::new();
    let mut all_slime_count = 0;
    
    if prep.chunk_count > 1 {
        let chunk_count_val = prep.chunk_count;
        
        for chunk_idx in 0..prep.chunk_count {
            let z_start = chunk_idx * (prep.chunk_size / prep.x_count);
            let z_end = std::cmp::min(z_start + (prep.chunk_size / prep.x_count), prep.z_count);
            let current_chunk_rows = z_end - z_start;
            
            progress_display.lock().unwrap().start_chunk(chunk_idx);
            log_writer.write_chunk_start(chunk_idx, chunk_count_val);
            
            let progress_counter = Arc::new(AtomicUsize::new(0));
            let counter_clone = Arc::clone(&progress_counter);
            let mut log_clone = log_writer.clone();
            let display_clone = Arc::clone(&progress_display);
            let chunk_idx_val = chunk_idx;
            
            let start_time = Instant::now();
            let handle = thread::spawn(move || {
                while counter_clone.load(Ordering::Relaxed) < current_chunk_rows {
                    let count = counter_clone.load(Ordering::Relaxed);
                    let percent = (count as f64 / current_chunk_rows as f64) * 100.0;
                    let elapsed = start_time.elapsed();
                    
                    display_clone.lock().unwrap().update_grid(percent, elapsed);
                    log_clone.write_chunk_update(chunk_idx_val, chunk_count_val, percent, elapsed);
                    
                    thread::sleep(Duration::from_millis(500));
                }
            });
            
            let chunk = grid::generate_grid_chunk_with_progress(config, prep, z_start, z_end, &progress_counter);
            
            handle.join().unwrap();
            
            let grid_elapsed = start_time.elapsed();
            progress_display.lock().unwrap().finish_grid(grid_elapsed);
            log_writer.write_chunk_finish(chunk_idx, chunk_count_val, grid_elapsed);
            
            all_slime_count += chunk.iter().filter(|&&x| x == 1).count();
            
            let half_size = config.count_size / 2;
            let chunk_start_block_z = prep.min_block_z + z_start as i32;
            let chunk_end_block_z = prep.min_block_z + z_end as i32 - 1;
            let valid_start_z = std::cmp::max(chunk_start_block_z + half_size, prep.min_block_z);
            let valid_end_z = std::cmp::min(chunk_end_block_z - half_size, prep.max_block_z);
            let process_total = ((valid_end_z - valid_start_z + 1) as usize) * prep.x_count;
            
            let process_progress = Arc::new(AtomicUsize::new(0));
            let process_counter_clone = Arc::clone(&process_progress);
            let process_display_clone = Arc::clone(&progress_display);
            let mut process_log_clone = log_writer.clone();
            let grid_elapsed_clone = grid_elapsed;
            
            let process_start = Instant::now();
            let process_handle = thread::spawn(move || {
                while process_counter_clone.load(Ordering::Relaxed) < process_total {
                    let count = process_counter_clone.load(Ordering::Relaxed);
                    let percent = (count as f64 / process_total as f64) * 100.0;
                    let elapsed = process_start.elapsed();
                    
                    process_display_clone.lock().unwrap().update_process(percent, elapsed);
                    process_log_clone.write_process_update(chunk_idx, chunk_count_val, grid_elapsed_clone, percent, elapsed);
                    
                    thread::sleep(Duration::from_millis(500));
                }
            });
            
            let chunk_results = counter::count_slime_chunks_chunk_with_progress(prep, &chunk, z_start, z_end, config.count_shape, config.count_size, config.count_target, &process_progress);
            
            process_handle.join().unwrap();
            
            let process_elapsed = process_start.elapsed();
            progress_display.lock().unwrap().finish_process(process_elapsed);
            log_writer.write_process_finish(chunk_idx, chunk_count_val, grid_elapsed, process_elapsed);
            
            for result in chunk_results {
                if all_results.len() < config.count_target {
                    all_results.push(Reverse(result));
                } else if let Some(Reverse(top)) = all_results.peek() {
                    if result.slime_count > top.slime_count || 
                       (result.slime_count == top.slime_count && result.distance_sq < top.distance_sq) {
                        all_results.pop();
                        all_results.push(Reverse(result));
                    }
                }
            }
        }
        
        progress_display.lock().unwrap().finish_all();
        log_writer.write_progress_finish();
    } else {
        let progress_counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&progress_counter);
        let mut log_clone = log_writer.clone();
        let display_clone = Arc::clone(&progress_display);
        let z_count_val = prep.z_count;
        
        progress_display.lock().unwrap().start_chunk(0);
        log_writer.write_chunk_start(0, 1);
        
        let start_time = Instant::now();
        let handle = thread::spawn(move || {
            while counter_clone.load(Ordering::Relaxed) < z_count_val {
                let count = counter_clone.load(Ordering::Relaxed);
                let percent = (count as f64 / z_count_val as f64) * 100.0;
                let elapsed = start_time.elapsed();
                
                display_clone.lock().unwrap().update_grid(percent, elapsed);
                log_clone.write_chunk_update(0, 1, percent, elapsed);
                
                thread::sleep(Duration::from_millis(500));
            }
        });
        
        let grid = grid::generate_grid_with_progress(config, prep, &progress_counter);
        
        handle.join().unwrap();
        
        let grid_elapsed = start_time.elapsed();
        progress_display.lock().unwrap().finish_grid(grid_elapsed);
        log_writer.write_chunk_finish(0, 1, grid_elapsed);
        
        all_slime_count = grid.iter().filter(|&&x| x == 1).count();
        
        let process_start = Instant::now();
        let results = counter::count_slime_chunks(prep, &grid, config.count_shape, config.count_size, config.count_target);
        let process_elapsed = process_start.elapsed();
        
        for result in results {
            all_results.push(Reverse(result));
        }
        
        progress_display.lock().unwrap().finish_process(process_elapsed);
        progress_display.lock().unwrap().finish_all();
        log_writer.write_progress_finish();
        
        if config.output_map {
            let path = grid_output::output_grid_file(config, prep, &grid)?;
            print_grid_file_written(config, &path);
        }
    }
    
    *total_slime_count = all_slime_count;
    
    let mut all_results: Vec<counter::CountResult> = all_results.into_iter().map(|Reverse(r)| r).collect();
    all_results.sort_by(|a, b| {
        b.slime_count.cmp(&a.slime_count)
            .then_with(|| a.distance_sq.cmp(&b.distance_sq))
    });
    
    if config.output_count {
        let count_path = count_output::output_count_file(config, prep, &all_results)?;
        print_count_file_written(config, &count_path);
        log_writer.write_count_file_written(&count_path);
    }
    
    print_count_summary(config, &all_results);
    log_writer.write_count_summary(&all_results);
    
    Ok(())
}
