/// 史莱姆区块判定库：完全复现 Java 版 Minecraft 的史莱姆区块算法

/// 判断指定坐标的区块是否为史莱姆区块
/// - world_seed: 世界种子
/// - chunk_x: 区块 X 坐标
/// - chunk_z: 区块 Z 坐标
/// - 返回: true 表示是史莱姆区块
#[inline(always)]
pub fn is_slime_chunk(world_seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
    let cx = chunk_x as i64;
    let cz = chunk_z as i64;

    let part1 = cx * cx * 4987142i64;
    let part2 = cx * 5947611i64;
    let part3 = cz * cz * 4392871i64;
    let part4 = cz * 389711i64;

    let seed = world_seed
        .wrapping_add(part1)
        .wrapping_add(part2)
        .wrapping_add(part3)
        .wrapping_add(part4)
        ^ 987234911i64;

    const MULTIPLIER: i64 = 0x5DEECE66D;
    const ADDEND: i64 = 0xB;
    const MASK: i64 = (1i64 << 48) - 1;

    let mut rng_seed = (seed ^ MULTIPLIER) & MASK;
    rng_seed = (rng_seed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND)) & MASK;
    let bits = (rng_seed >> (48 - 31)) as i32;
    
    bits % 10 == 0
}
