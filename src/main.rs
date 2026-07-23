/// 主程序：仅作为数据中转站，不执行任何计算或格式化
mod slime_lib;
mod preprocess;
mod matcher;
mod matcher_output;
mod info;
mod timer;
mod stats;

use anyhow::Result;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use preprocess::{Preprocessed, OutputMode};

fn main() -> Result<()> {
    let mut timer = timer::Timer::new();

    // ========== 阶段1：预处理 ==========
    timer.start_stage();
    // 发送配置文件路径到预处理模块，接收预处理结果
    let pre = preprocess::Preprocessed::from_file("config.toml")?;
    timer.end_preprocess();

    // ========== 阶段2：计算 ==========
    timer.start_stage();
    // 发送预处理结果到计算模块，接收匹配结果和统计数据
    let (matches, total_blocks) = matcher::find_matches(&pre);
    timer.end_calculation();

    // ========== 阶段3：格式化 ==========
    // 发送数据到格式化模块，接收格式化后的字符串
    let info_str = info::generate_info(&pre);
    let result_str = matcher_output::format_results(&matches, pre.center_x, pre.center_z);

    // ========== 阶段4：输出（main直接合并info和result并输出） ==========
    timer.start_stage();
    let output_content = format!("{}{}", info_str, result_str);
    output(&pre, &output_content, false)?;
    timer.end_output();
    timer.end_total();

    // ========== 阶段5：统计信息 ==========
    // 获取统计信息并输出
    let stats = stats::Stats::new(timer.times(), total_blocks, matches.len());
    let stats_str = stats.generate_stats_string();
    output(&pre, &stats_str, true)?;

    Ok(())
}

/// 输出内容到控制台和/或文件
fn output(pre: &Preprocessed, content: &str, append: bool) -> Result<()> {
    match pre.output_mode {
        OutputMode::Console => {
            print!("{}", content);
        }
        OutputMode::File => {
            let path = pre.output_path.as_ref().expect("文件模式需指定路径");
            if append {
                let mut f = OpenOptions::new().append(true).open(path)?;
                f.write_all(content.as_bytes())?;
            } else {
                let mut f = File::create(path)?;
                f.write_all(content.as_bytes())?;
            }
        }
        OutputMode::Both => {
            print!("{}", content);
            let path = pre.output_path.as_ref().expect("文件模式需指定路径");
            if append {
                let mut f = OpenOptions::new().append(true).open(path)?;
                f.write_all(content.as_bytes())?;
            } else {
                let mut f = File::create(path)?;
                f.write_all(content.as_bytes())?;
            }
        }
    }
    Ok(())
}
