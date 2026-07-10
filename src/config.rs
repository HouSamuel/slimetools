#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Check,
    Match,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalMode {
    None,
    Basic,
    Full,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub world_seed: i128,
    
    pub center_block_x: i32,
    
    pub center_block_z: i32,
    
    pub radius: i32,
    
    pub mode: Mode,
    
    pub memory_limit_gib: f64,
    
    pub pattern: Option<&'static [&'static [u8]]>,
    
    pub match_target: usize,
    
    pub output_map: bool,
    
    pub output_match: bool,
    
    pub output_log: bool,
    
    pub terminal_mode: TerminalMode,
    
    pub secure_mode: bool,
    
    pub challenge_code: Option<u32>,
    
    pub progress_update_interval: f64,
}

const PATTERN_ROWS: &[&[u8]] = &[
    &[1, 1, 1],
    &[1, 1, 1],
    &[1, 1, 1],
];

pub const CONFIG: Config = Config {
    world_seed: 20260627,
    center_block_x: 0,
    center_block_z: 0,
    radius: 100,
    mode: Mode::Check,
    memory_limit_gib: 6.0,
    
    pattern: Some(PATTERN_ROWS),
    match_target: 0,
    
    output_map: true,
    output_match: true,
    output_log: true,
    terminal_mode: TerminalMode::Full,
    progress_update_interval: 1.0,
    
    secure_mode: false,
    challenge_code: None,
};
