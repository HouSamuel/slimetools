/// 匹配结果输出模块：格式化匹配结果为字符串
/// 
/// 负责将匹配结果格式化为可读字符串，包含序号、区块坐标、世界坐标和距离信息，
/// 并按距离中心的距离排序。支持自动排版对齐。

/// 格式化匹配结果为可读字符串
/// - matches: 匹配位置列表
/// - center_x, center_z: 中心区块坐标（用于计算距离）
/// - 返回: 格式化的结果字符串
pub fn format_results(matches: &[(i32, i32)], center_x: i32, center_z: i32) -> String {
    let mut s = String::from("[Result]\n");
    
    if matches.is_empty() {
        s.push_str("未找到匹配的图案\n");
    } else {
        // 复制并按距离排序
        let mut sorted = matches.to_vec();
        sorted.sort_by_key(|&(x, z)| {
            let dx = (x - center_x).abs();
            let dz = (z - center_z).abs();
            dx + dz
        });
        
        let total = sorted.len();
        
        // 计算各列宽度（确保负号被包含在内）
        let idx_width = total.to_string().len();
        
        // 区块坐标宽度：取所有坐标字符串的最大长度（包含负号）
        let chunk_coord_width = sorted.iter()
            .flat_map(|&(x, z)| [x, z])
            .map(|c| c.to_string().len())
            .max().unwrap_or(3);
        
        // 世界坐标宽度：取所有世界坐标字符串的最大长度（包含负号）
        let world_coord_width = sorted.iter()
            .flat_map(|&(x, z)| [x * 16, x * 16 + 16, z * 16, z * 16 + 16])
            .map(|c| c.to_string().len())
            .max().unwrap_or(4);
        
        // 距离宽度
        let distance_width = sorted.iter()
            .map(|&(x, z)| ((x - center_x).abs() + (z - center_z).abs()).to_string().len())
            .max().unwrap_or(2);
        
        s.push_str(&format!("找到 {} 个匹配（按距离中心排序）：\n\n", total));
        
        // 表头（使用固定格式，确保与数据行对齐）
        s.push_str(&format!(
            "[{:>idx_w$}/{}]  ({:>cc_w$}, {:>cc_w$})  =>  x:[{:>wc_w$}, {:>wc_w$}) z:[{:>wc_w$}, {:>wc_w$})  距离: {:>d_w$} 区块\n",
            "序", "总数", "区块X", "区块Z", "世界X", "世界X+", "世界Z", "世界Z+", "距离",
            idx_w = idx_width,
            cc_w = chunk_coord_width,
            wc_w = world_coord_width,
            d_w = distance_width
        ));
        
        // 分隔线长度
        let line_len = 
            idx_width * 2 + 5 +           // [idx/total]
            chunk_coord_width * 2 + 6 +   // (x, z)
            4 +                           // => 
            3 + world_coord_width * 2 + 3 + // x:[wx, wx+16)
            3 + world_coord_width * 2 + 3 + // z:[wz, wz+16)
            7 + distance_width + 4;       // 距离: n 区块
        
        s.push_str(&"-".repeat(line_len));
        s.push('\n');
        
        // 输出每行数据，所有列对齐
        for (idx, &(x, z)) in sorted.iter().enumerate() {
            let wx_start = x * 16;
            let wx_end = wx_start + 16;
            let wz_start = z * 16;
            let wz_end = wz_start + 16;
            let distance = (x - center_x).abs() + (z - center_z).abs();
            
            s.push_str(&format!(
                "[{:>idx_w$}/{}]  ({:>cc_w$}, {:>cc_w$})  =>  x:[{:>wc_w$}, {:>wc_w$}) z:[{:>wc_w$}, {:>wc_w$})  距离: {:>d_w$} 区块\n",
                idx + 1, total, x, z, wx_start, wx_end, wz_start, wz_end, distance,
                idx_w = idx_width,
                cc_w = chunk_coord_width,
                wc_w = world_coord_width,
                d_w = distance_width
            ));
        }
        
        s.push('\n');
    }
    
    s
}
