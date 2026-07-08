#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Check,
    Match,
    Count,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountShape {
    Square,
    Circle,
}

#[derive(Debug, Clone)]
pub struct Config {
    // === 全局基础配置 ===
    /// 世界种子
    pub world_seed: i128,
    
    /// 扫描中心区块 X 坐标（区块坐标）
    pub center_block_x: i32,
    
    /// 扫描中心区块 Z 坐标（区块坐标）
    pub center_block_z: i32,
    
    /// 扫描半径（区块），以中心区块为原点向四周扩展
    pub radius: i32,
    
    /// 运行模式：Check/Match/Count
    pub mode: Mode,
    
    /// 内存限制（GiB），达到后自动分块处理
    pub memory_limit_gib: Option<f64>,
    
    // === Match 模式专属配置 ===
    /// 匹配图案（二维数组形式），0=任意区块，1=必须是史莱姆区块，2=必须不是史莱姆区块
    /// 仅 Match 模式有效
    pub pattern: Option<&'static [&'static [u8]]>,
    
    /// 匹配目标数量，0=全部查找，其他值=找到指定数量后停止
    /// 仅 Match 模式有效
    pub match_target: usize,
    
    // === Count 模式专属配置 ===
    /// 计数区域形状：Square（正方形）/ Circle（圆形）
    /// 仅 Count 模式有效
    pub count_shape: CountShape,
    
    /// 计数区域尺寸：正方形边长或圆形半径
    /// 仅 Count 模式有效
    pub count_size: i32,
    
    /// 计数目标数量，取史莱姆区块最多的前 N 个区域
    /// 仅 Count 模式有效
    pub count_target: usize,
    
    // === 输出配置 ===
    /// 是否输出网格文件（{seed}_map.txt）
    pub output_map: bool,
    
    /// 是否输出匹配结果文件（{seed}_match.txt）
    pub output_match: bool,
    
    /// 是否输出计数结果文件（{seed}_count.txt）
    pub output_count: bool,
    
    /// 是否输出日志文件（{seed}_log.txt）
    pub output_log: bool,
    
    /// 是否在终端输出信息
    pub terminal_output: bool,
}

// === 用户输入参数 ===

/// 匹配图案（二维矩阵形式）
/// 0 = 任意区块，1 = 必须是史莱姆区块，2 = 必须不是史莱姆区块
/// 仅在 Match 模式下生效，修改后无需手动更新长宽，自动推导
const PATTERN_ROWS: &[&[u8]] = &[
    &[1, 1],
    &[1, 1],
];

/// 全局配置实例
/// 用户只需修改这里的参数即可，无需修改其他文件
pub const CONFIG: Config = Config {
    // === 全局基础配置 ===
    world_seed: 20260627,
    center_block_x: 0,
    center_block_z: 0,
    radius: 100,
    mode: Mode::Match,
    memory_limit_gib: None,
    
    // === Match 模式专属配置 ===
    pattern: Some(PATTERN_ROWS),
    match_target: 0,
    
    // === Count 模式专属配置 ===
    count_shape: CountShape::Square,
    count_size: 5,
    count_target: 10,
    
    // === 输出配置 ===
    output_map: false,
    output_match: true,
    output_count: true,
    output_log: true,
    terminal_output: true,
};