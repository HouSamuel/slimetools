/// 预处理模块：读取配置文件，验证输入合法性，计算扫描范围
use serde::Deserialize;
use std::fs;
use anyhow::{Context, Result};
use crate::pattern::Pattern;

/// 原始配置：从 TOML 文件解析得到
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RawConfig {
    pub seed: i64,              // 世界种子
    pub center_x: i32,          // 中心区块 X 坐标
    pub center_z: i32,          // 中心区块 Z 坐标
    pub radius: u32,            // 扫描半径（区块数）
    pub pattern: Vec<Vec<u8>>,  // 匹配图案（0:非史莱姆, 1:史莱姆, 2:忽略）
    pub limit: usize,           // 最大匹配数（0表示无限制）
    pub output_mode: String,    // 输出模式（console/file/both）
    pub output_path: Option<String>, // 输出文件路径
}

/// 输出模式枚举
#[derive(Debug)]
pub enum OutputMode {
    Console,  // 仅控制台输出
    File,     // 仅文件输出
    Both,     // 控制台和文件同时输出
}

impl From<&str> for OutputMode {
    /// 从字符串解析输出模式
    fn from(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "file" => OutputMode::File,
            "both" => OutputMode::Both,
            _ => OutputMode::Console,
        }
    }
}

/// 预处理结果：包含计算后的扫描范围和配置信息
#[derive(Debug)]
#[allow(dead_code)]
pub struct Preprocessed {
    pub seed: i64,              // 世界种子
    pub center_x: i32,          // 中心区块 X 坐标
    pub center_z: i32,          // 中心区块 Z 坐标
    pub radius: u32,            // 扫描半径（区块数）
    pub x_start: i32,           // X 方向扫描起始坐标
    pub x_end: i32,             // X 方向扫描结束坐标
    pub z_start: i32,           // Z 方向扫描起始坐标
    pub z_end: i32,             // Z 方向扫描结束坐标
    pub pattern: Pattern,       // 预处理后的图案（SWAR优化）
    pub limit: usize,           // 最大匹配数
    pub pat_h: usize,           // 图案高度
    pub pat_w: usize,           // 图案宽度
    pub output_mode: OutputMode,// 输出模式
    pub output_path: Option<String>, // 输出文件路径
    pub cache_x0: i32,          // 缓存范围起始 X（扩展后）
    pub cache_x1: i32,          // 缓存范围结束 X（扩展后）
    pub cache_z0: i32,          // 缓存范围起始 Z（扩展后）
    pub cache_z1: i32,          // 缓存范围结束 Z（扩展后）
}

impl Preprocessed {
    /// 从配置文件读取并预处理
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("无法读取配置文件 {}", path))?;
        let raw: RawConfig = toml::from_str(&content)
            .with_context(|| "解析配置文件失败")?;

        // 预处理图案（SWAR优化）
        let pattern = Pattern::new(&raw.pattern)
            .with_context(|| "图案预处理失败")?;

        let (pat_h, pat_w) = (pattern.height, pattern.width);
        let radius = raw.radius as i32;
        let center_x = raw.center_x;
        let center_z = raw.center_z;

        // 计算对称扫描范围：[-radius, radius]
        let x_start = center_x - radius;
        let x_end = center_x + radius;
        let z_start = center_z - radius;
        let z_end = center_z + radius;

        // 确定输出模式和路径（强制使用种子号命名）
        let output_mode = OutputMode::from(raw.output_mode.as_str());
        let output_path = match output_mode {
            OutputMode::File | OutputMode::Both => {
                Some(format!("{}_match_output.txt", raw.seed))
            }
            OutputMode::Console => None,
        };

        // 缓存范围（扩展以容纳图案越界）
        let cache_x0 = x_start;
        let cache_x1 = x_end + pat_w as i32 - 1;
        let cache_z0 = z_start;
        let cache_z1 = z_end + pat_h as i32 - 1;

        Ok(Preprocessed {
            seed: raw.seed,
            center_x,
            center_z,
            radius: raw.radius,
            x_start,
            x_end,
            z_start,
            z_end,
            pattern,
            limit: raw.limit,
            pat_h,
            pat_w,
            output_mode,
            output_path,
            cache_x0,
            cache_x1,
            cache_z0,
            cache_z1,
        })
    }
}
