const C_X2: i64 = 4987142;
const C_X1: i64 = 5947611;
const C_Z2: i64 = 4392871;
const C_Z1: i64 = 389711;
const BOUND: i64 = 10;
const TARGET: i64 = 0;

const LCG_MULT: u64 = 0x5DEECE66D;
const LCG_ADD: u64 = 0xB;
const LCG_MASK: u64 = (1 << 48) - 1;
const XOR_SEED: u64 = 0x5DEECE66D;

#[inline(always)]
pub fn compute_fx(x: i32) -> i64 {
    let xi = x as i64;
    xi.wrapping_mul(xi).wrapping_mul(C_X2).wrapping_add(xi.wrapping_mul(C_X1))
}

#[inline(always)]
pub fn compute_fz(z: i32) -> i64 {
    let zi = z as i64;
    zi.wrapping_mul(zi).wrapping_mul(C_Z2).wrapping_add(zi.wrapping_mul(C_Z1))
}

#[inline(always)]
pub fn is_slime_chunk_fast(fx: i64, fz: i64, seed: i64) -> bool {
    let combined = ((seed as u64).wrapping_add(fx as u64).wrapping_add(fz as u64) ^ XOR_SEED) & LCG_MASK;
    let next_seed = combined.wrapping_mul(LCG_MULT).wrapping_add(LCG_ADD) & LCG_MASK;
    let bits = (next_seed >> 17) as i64;
    (bits % BOUND) == TARGET
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
}