use std::time::Duration;

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub prep_time: Duration,
    pub grid_time: Duration,
    pub process_time: Duration,
    pub output_time: Duration,
    pub total_time: Duration,
    pub slime_count: usize,
    pub total_blocks: usize,
}

impl Stats {
    pub fn blocks_per_second(&self) -> f64 {
        if self.total_time.as_secs_f64() > 0.0 {
            (self.total_blocks as f64) / self.total_time.as_secs_f64()
        } else {
            0.0
        }
    }
    
    pub fn slime_percentage(&self) -> f64 {
        if self.total_blocks > 0 {
            (self.slime_count as f64 / self.total_blocks as f64) * 100.0
        } else {
            0.0
        }
    }
    
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("[Stats]\n");
        report.push_str(&format!("预处理耗时: {:?}\n", self.prep_time));
        report.push_str(&format!("网格生成耗时: {:?}\n", self.grid_time));
        report.push_str(&format!("处理耗时: {:?}\n", self.process_time));
        report.push_str(&format!("输出耗时: {:?}\n", self.output_time));
        report.push_str(&format!("总耗时: {:?}\n", self.total_time));
        report.push_str(&format!("处理速度: {:.0} 区块/秒\n", self.blocks_per_second()));
        report.push_str(&format!("史莱姆区块数: {} ({:.2}%)\n", 
            self.slime_count, self.slime_percentage()));
        report
    }
}
