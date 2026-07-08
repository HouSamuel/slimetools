use std::time::Instant;

mod config;
mod slime_chunk;
mod preprocess;
mod grid;
mod matcher;
mod counter;
mod grid_output;
mod match_output;
mod count_output;
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
    
    let grid_start = Instant::now();
    let grid = grid::generate_grid(config, &prep);
    let slime_count = grid.iter().filter(|&&x| x == 1).count();
    let grid_time = grid_start.elapsed();
    
    print_info(config, &prep);
    print_chunking_info(config, &prep);
    
    let mut log_writer = log::LogWriter::new(config)?;
    log_writer.write_info(config, &prep)?;
    
    print_grid_start(config);
    print_grid_done(config);
    
    let process_start = Instant::now();
    
    match config.mode {
        config::Mode::Check => {
            let process_time = process_start.elapsed();
            
            let output_start = Instant::now();
            
            if config.output_map {
                let path = grid_output::output_grid_file(config, &prep, &grid)?;
                print_grid_file_written(config, &path);
            }
            
            let output_time = output_start.elapsed();
            let total_time = total_start.elapsed();
            
            print_stats(config, prep_time, grid_time, process_time, output_time, total_time, slime_count, prep.total_blocks);
            log_writer.write_stats(prep_time, grid_time, process_time, output_time, total_time, slime_count, prep.total_blocks)?;
            log_writer.flush()?;
        }
        
        config::Mode::Match => {
            print_match_start(config);
            
            let matches = matcher::find_matches(config, &prep, &grid);
            
            print_match_done(config);
            
            let process_time = process_start.elapsed();
            
            let output_start = Instant::now();
            
            if config.output_match {
                let match_path = match_output::output_match_file(config, &prep, &matches)?;
                print_match_file_written(config, &match_path);
            }
            
            print_match_summary(config, &matches);
            
            if config.output_map {
                let grid_path = grid_output::output_grid_file(config, &prep, &grid)?;
                print_grid_file_written(config, &grid_path);
            }
            
            let output_time = output_start.elapsed();
            let total_time = total_start.elapsed();
            
            print_stats(config, prep_time, grid_time, process_time, output_time, total_time, slime_count, prep.total_blocks);
            log_writer.write_stats(prep_time, grid_time, process_time, output_time, total_time, slime_count, prep.total_blocks)?;
            log_writer.flush()?;
        }
        
        config::Mode::Count => {
            print_count_start(config);
            
            let results = counter::count_slime_chunks(&prep, &grid, config.count_shape, config.count_size, config.count_target);
            
            print_count_done(config);
            
            let process_time = process_start.elapsed();
            
            let output_start = Instant::now();
            
            if config.output_count {
                let count_path = count_output::output_count_file(config, &prep, &results)?;
                print_count_file_written(config, &count_path);
            }
            
            print_count_summary(config, &results);
            
            if config.output_map {
                let grid_path = grid_output::output_grid_file(config, &prep, &grid)?;
                print_grid_file_written(config, &grid_path);
            }
            
            let output_time = output_start.elapsed();
            let total_time = total_start.elapsed();
            
            print_stats(config, prep_time, grid_time, process_time, output_time, total_time, slime_count, prep.total_blocks);
            log_writer.write_stats(prep_time, grid_time, process_time, output_time, total_time, slime_count, prep.total_blocks)?;
            log_writer.flush()?;
        }
    }
    
    Ok(())
}