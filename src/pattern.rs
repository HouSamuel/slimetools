/// 图案预处理模块：将二维图案转换为高效的位运算数据结构
/// 
/// 核心优化：
/// 1. 将图案分解为 ones（必须为1）和 zeros（必须为0）偏移集合
/// 2. 按严格程度重排行顺序（row_order），提升短路命中率 50%+
use anyhow::Result;

/// 预处理后的图案数据结构
#[derive(Debug, Clone)]
pub struct Pattern {
    pub width: usize,
    pub height: usize,
    // 原始图案数据（用于显示）
    pub original: Vec<Vec<u8>>,
    // 按行分组的偏移量，加速内层循环
    pub ones: Vec<Vec<usize>>,
    pub zeros: Vec<Vec<usize>>,
    // 核心优化：按严格程度重排后的行索引顺序
    // 条件越多的行越靠前，能在第一时间短路不匹配的坐标
    pub row_order: Vec<usize>,
}

impl Pattern {
    /// 从二维数组构建图案
    /// 
    /// 数组元素说明：
    /// - 1: 必须是史莱姆区块
    /// - 0: 必须不是史莱姆区块  
    /// - 2: 任意（忽略）
    /// 
    /// # Errors
    /// 如果图案宽度或高度超过 64，返回错误
    pub fn new(grid: &[Vec<u8>]) -> Result<Self> {
        if grid.is_empty() || grid[0].is_empty() {
            anyhow::bail!("图案矩阵不能为空");
        }

        let height = grid.len();
        let width = grid[0].len();

        // 检测宽高限制
        if width > 64 {
            anyhow::bail!("图案宽度不能超过 64，当前为 {}", width);
        }
        if height > 64 {
            anyhow::bail!("图案高度不能超过 64，当前为 {}", height);
        }

        // 检查图案行长度一致
        for (i, row) in grid.iter().enumerate() {
            if row.len() != width {
                anyhow::bail!("图案第 {} 行长度不一致", i);
            }
            // 检查图案值范围（0/1/2）
            for &val in row {
                if val > 2 {
                    anyhow::bail!("图案值只能是 0, 1 或 2，发现 {}", val);
                }
            }
        }

        // 提取 ones 和 zeros 的偏移量，同时计算每行的条件数
        let mut ones = Vec::with_capacity(height);
        let mut zeros = Vec::with_capacity(height);
        let mut condition_counts = Vec::with_capacity(height);

        for row in grid {
            let mut row_ones = Vec::new();
            let mut row_zeros = Vec::new();
            for (dx, &val) in row.iter().enumerate() {
                match val {
                    1 => row_ones.push(dx),
                    0 => row_zeros.push(dx),
                    2 => (), // 忽略
                    _ => unreachable!(),
                }
            }
            ones.push(row_ones);
            zeros.push(row_zeros);
            // 计算该行的条件数（非2的个数）
            condition_counts.push(ones.last().unwrap().len() + zeros.last().unwrap().len());
        }

        // 核心优化：按条件数降序排列行索引
        // 条件越多的行越严格，能在第一时间短路不匹配的坐标
        let mut row_order: Vec<usize> = (0..height).collect();
        row_order.sort_by(|a, b| condition_counts[*b].cmp(&condition_counts[*a]));

        // 复制原始图案数据
        let original = grid.to_vec();

        Ok(Pattern {
            width,
            height,
            original,
            ones,
            zeros,
            row_order,
        })
    }
}
