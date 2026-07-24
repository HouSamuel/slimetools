/// 统计模块：生成性能统计信息
use crate::timer::StageTimes;

/// 统计数据：包含计时和计算结果
pub struct Stats {
    times: StageTimes,      // 各阶段耗时
    total_blocks: u64,      // 扫描区块总数（使用 u64 避免溢出）
    match_count: usize,     // 匹配结果数
}

impl Stats {
    /// 创建统计对象
    pub fn new(times: StageTimes, total_blocks: u64, match_count: usize) -> Self {
        Stats {
            times,
            total_blocks,
            match_count,
        }
    }

    /// 生成格式化的统计信息字符串
    pub fn generate_stats_string(&self) -> String {
        let calc_duration = self.times.calculation.as_secs_f64();
        
        // 计算效率 = 总区块数 / 计算耗时（只看计算阶段的时间）
        let efficiency = if calc_duration > 0.0 {
            self.total_blocks as f64 / calc_duration
        } else {
            0.0
        };

        // 将效率数字格式化为带千位分隔符的字符串
        let efficiency_str = format!("{:.2}", efficiency);
        let efficiency_formatted = Self::add_thousand_separator(&efficiency_str);

        // 将区块总数格式化为带千位分隔符的字符串
        let total_blocks_formatted = Self::add_thousand_separator(&self.total_blocks.to_string());

        format!(
            "[Statistics]\n\
             预处理耗时: {:.2?}\n\
             计算耗时: {:.2?}\n\
             输出耗时: {:.2?}\n\
             总耗时: {:.2?}\n\
             扫描区块总数: {}\n\
             计算效率: {} 区块/秒\n\
             匹配结果数: {}\n",
            self.times.preprocess,
            self.times.calculation,
            self.times.output,
            self.times.total,
            total_blocks_formatted,
            efficiency_formatted,
            self.match_count
        )
    }

    /// 为数字字符串添加千位分隔符
    fn add_thousand_separator(s: &str) -> String {
        let parts: Vec<&str> = s.split('.').collect();
        let integer_part = parts[0];
        let decimal_part = if parts.len() > 1 { parts[1] } else { "" };

        let mut result = String::new();
        let mut count = 0;

        // 从右向左遍历整数部分
        for c in integer_part.chars().rev() {
            if count > 0 && count % 3 == 0 {
                result.push(',');
            }
            result.push(c);
            count += 1;
        }

        // 反转回来
        result = result.chars().rev().collect();

        // 添加小数部分
        if !decimal_part.is_empty() {
            result.push('.');
            result.push_str(decimal_part);
        }

        result
    }
}
