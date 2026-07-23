/// 信息输出模块：生成程序运行信息（配置、范围等）
use crate::preprocess::Preprocessed;

/// 生成运行信息字符串
/// - pre: 预处理结果
/// - 返回: 格式化的信息字符串
pub fn generate_info(pre: &Preprocessed) -> String {
    let mut s = String::new();
    s.push_str("[Info]\n");

    // 世界种子
    s.push_str(&format!("世界种子: {}\n", pre.seed));

    // 中心区块坐标
    let (cx, cz) = (pre.center_x, pre.center_z);
    s.push_str(&format!("中心区块坐标: ({}, {})\n", cx, cz));

    // 中心区块对应的世界坐标范围
    let wx_start = cx * 16;
    let wx_end = wx_start + 16;
    let wz_start = cz * 16;
    let wz_end = wz_start + 16;
    s.push_str(&format!("世界坐标: x:[{}, {}) z:[{}, {})\n", wx_start, wx_end, wz_start, wz_end));

    // 扫描半径
    s.push_str(&format!("扫描半径: {} chunks\n", pre.radius));

    // 扫描区块坐标范围
    let (x_start, x_end) = (pre.x_start, pre.x_end);
    let (z_start, z_end) = (pre.z_start, pre.z_end);
    s.push_str(&format!("区块坐标范围: x:({}, {}) z:({}, {})\n", x_start, x_end, z_start, z_end));

    // 完整世界坐标范围（缓存范围）
    let wx_start2 = pre.cache_x0 * 16;
    let wx_end2 = (pre.cache_x1 + 1) * 16;
    let wz_start2 = pre.cache_z0 * 16;
    let wz_end2 = (pre.cache_z1 + 1) * 16;
    s.push_str(&format!("世界坐标范围: x:[{}, {}) z:[{}, {})\n", wx_start2, wx_end2, wz_start2, wz_end2));

    // 匹配图案
    s.push_str("图案:\n");
    for (i, row) in pre.pattern.iter().enumerate() {
        let row_str: String = row.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(" ");
        s.push_str(&format!("行 {}: {}\n", i + 1, row_str));
    }

    s
}
