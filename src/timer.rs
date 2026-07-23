/// 计时器模块：测量各阶段耗时
use std::time::{Instant, Duration};

/// 各阶段耗时统计
#[derive(Debug, Clone, Copy)]
pub struct StageTimes {
    pub preprocess: Duration,  // 预处理耗时
    pub calculation: Duration, // 计算耗时
    pub output: Duration,      // 输出耗时
    pub total: Duration,       // 总耗时
}

/// 计时器：支持多阶段计时
pub struct Timer {
    start_total: Instant,   // 程序启动时间
    start_stage: Instant,   // 当前阶段启动时间
    times: StageTimes,      // 已记录的各阶段耗时
}

impl Timer {
    /// 创建新计时器
    pub fn new() -> Self {
        let now = Instant::now();
        Timer {
            start_total: now,
            start_stage: now,
            times: StageTimes {
                preprocess: Duration::ZERO,
                calculation: Duration::ZERO,
                output: Duration::ZERO,
                total: Duration::ZERO,
            },
        }
    }

    /// 开始新的计时阶段
    pub fn start_stage(&mut self) {
        self.start_stage = Instant::now();
    }

    /// 结束预处理阶段，记录耗时
    pub fn end_preprocess(&mut self) {
        self.times.preprocess = self.start_stage.elapsed();
    }

    /// 结束计算阶段，记录耗时
    pub fn end_calculation(&mut self) {
        self.times.calculation = self.start_stage.elapsed();
    }

    /// 结束输出阶段，记录耗时
    pub fn end_output(&mut self) {
        self.times.output = self.start_stage.elapsed();
    }

    /// 结束总计时
    pub fn end_total(&mut self) {
        self.times.total = self.start_total.elapsed();
    }

    /// 获取所有阶段耗时统计
    pub fn times(&self) -> StageTimes {
        self.times
    }
}
