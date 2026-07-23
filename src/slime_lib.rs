/// 史莱姆区块判定库：完全复现 Java 版 Minecraft 的史莱姆区块算法
///
/// 史莱姆区块是 Minecraft 中史莱姆自然生成的必要条件（除沼泽生物群系外）。
/// 判定算法基于世界种子和区块坐标，通过伪随机数生成器决定是否为史莱姆区块。

/// 判断指定坐标的区块是否为史莱姆区块
/// - world_seed: 世界种子
/// - chunk_x: 区块 X 坐标
/// - chunk_z: 区块 Z 坐标
/// - 返回: true 表示是史莱姆区块
pub fn is_slime_chunk(world_seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
    // 计算初始种子：混合世界种子和区块坐标
    let part1 = (chunk_x.wrapping_mul(chunk_x).wrapping_mul(4_987_142)) as i64;
    let part2 = (chunk_x.wrapping_mul(5_947_611)) as i64;
    let part3 = ((chunk_z.wrapping_mul(chunk_z)) as i64).wrapping_mul(4_392_871);
    let part4 = (chunk_z.wrapping_mul(389_711)) as i64;

    let seed = world_seed
        .wrapping_add(part1)
        .wrapping_add(part2)
        .wrapping_add(part3)
        .wrapping_add(part4)
        ^ 987_234_911;

    // Java Random 算法参数
    const MULTIPLIER: i64 = 0x5DEECE66D;
    const ADDEND: i64 = 0xB;
    const MASK: i64 = (1i64 << 48) - 1;

    let mut rng_seed = (seed ^ MULTIPLIER) & MASK;

    // 生成下一个随机数（取高 31 位）
    fn next(seed: &mut i64, bits: i32) -> i32 {
        *seed = (seed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND)) & MASK;
        (*seed >> (48 - bits)) as i32
    }

    // 生成 0-9 的随机数，若为 0 则是史莱姆区块（概率 10%）
    let bits = next(&mut rng_seed, 31);
    bits % 10 == 0
}
