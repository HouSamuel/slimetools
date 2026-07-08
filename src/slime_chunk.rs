const C_X2: i32 = 4987142;
const C_X1: i32 = 5947611;
const C_Z2: i64 = 4392871;
const C_Z1: i32 = 389711;
const XOR_MASK: i64 = 987234911;
const BOUND: i32 = 10;
const TARGET: i32 = 0;

const LCG_MULT: u64 = 0x5DEECE66D;
const LCG_ADD: u64 = 0xB;
const LCG_MASK: u64 = (1 << 48) - 1;
const XOR_SEED: u64 = 0x5DEECE66D;

#[inline(always)]
pub fn compute_fx(x: i32) -> i64 {
    let xi = x;
    let term1 = (xi.wrapping_mul(xi).wrapping_mul(C_X2)) as i64;
    let term2 = (xi.wrapping_mul(C_X1)) as i64;
    term1 + term2
}

#[inline(always)]
pub fn compute_fz(z: i32) -> i64 {
    let zi = z;
    let term3 = (zi.wrapping_mul(zi) as i64) * C_Z2;
    let term4 = (zi.wrapping_mul(C_Z1)) as i64;
    term3 + term4
}

#[inline(always)]
pub fn is_slime_chunk_fast(fx: i64, fz: i64, seed: i64) -> bool {
    let combined = (seed as u64)
        .wrapping_add(fx as u64)
        .wrapping_add(fz as u64) as i64;
    let lcg_seed = (((combined ^ XOR_MASK) as u64) ^ XOR_SEED) & LCG_MASK;
    let next_seed = lcg_seed.wrapping_mul(LCG_MULT).wrapping_add(LCG_ADD) & LCG_MASK;
    let bits = (next_seed >> 17) as i64;
    (bits % BOUND as i64) == TARGET as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_fx_zero() {
        assert_eq!(compute_fx(0), 0);
    }

    #[test]
    fn test_compute_fz_zero() {
        assert_eq!(compute_fz(0), 0);
    }

    #[test]
    fn test_is_slime_chunk_consistency() {
        let seed = 20260627i64;
        let fx = compute_fx(0);
        let fz = compute_fz(0);
        assert_eq!(is_slime_chunk_fast(fx, fz, seed), is_slime_chunk_fast(fx, fz, seed));
    }

    #[test]
    fn test_is_slime_chunk_fast_values() {
        let seed = 20260627i64;
        let fx = compute_fx(100);
        let fz = compute_fz(200);
        is_slime_chunk_fast(fx, fz, seed);
    }

    #[test]
    fn test_slime_chunk_java_compare() {
        let seed = 20260627i64;
        
        let test_cases = [
            (0, 0, false),
            (100, 200, true),
            (1000000, 500000, false),
            (12709, -21200, false),
            (-1000000, 1000000, false),
            (500000, 500000, false),
            (999999, 999999, true),
            (-999999, -999999, false),
            (100000, 100000, false),
            (-100000, -100000, false),
            (100, 100, true),
            (-100, -100, false),
            (500, 500, true),
            (-500, 500, false),
            (1000, 0, false),
            (0, 1000, false),
        ];
        
        for &(x, z, expected) in &test_cases {
            let fx = compute_fx(x);
            let fz = compute_fz(z);
            let result = is_slime_chunk_fast(fx, fz, seed);
            assert_eq!(result, expected, "x={}, z={}: expected {}, got {}", x, z, expected, result);
        }
    }
}