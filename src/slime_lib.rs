/// 判断是否为史莱姆区块，完全复现 Java 版行为
pub fn is_slime_chunk(world_seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
    let part1 = (chunk_x.wrapping_mul(chunk_x).wrapping_mul(4_987_142)) as i64;
    let part2 = (chunk_x.wrapping_mul(5_947_611)) as i64;
    let part3 = ((chunk_z.wrapping_mul(chunk_z)) as i64).wrapping_mul(4_392_871);
    let part4 = (chunk_z.wrapping_mul(389_711)) as i64;

    let seed = world_seed
        .wrapping_add(part1)
        .wrapping_add(part2)
        .wrapping_add(part3)
        .wrapping_add(part4)
        ^ 987_234_911;   // 直接异或，无需 wrapping

    const MULTIPLIER: i64 = 0x5DEECE66D;
    const ADDEND: i64 = 0xB;
    const MASK: i64 = (1i64 << 48) - 1;   // 明确指定 i64

    let mut rng_seed = (seed ^ MULTIPLIER) & MASK;

    fn next(seed: &mut i64, bits: i32) -> i32 {
        *seed = (seed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND)) & MASK;
        (*seed >> (48 - bits)) as i32
    }

    let bits = next(&mut rng_seed, 31);
    bits % 10 == 0
}