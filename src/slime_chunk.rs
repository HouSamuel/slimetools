const C_X2: i64 = 4987142;
const C_X1: i64 = 5947611;
const C_Z2: i64 = 4392871;
const C_Z1: i64 = 389711;
const XOR_MASK: i64 = 987234911;
const BOUND: i32 = 10;
const TARGET: i32 = 0;

const LCG_MULT: u64 = 0x5DEECE66D;
const LCG_ADD: u64 = 0xB;
const LCG_MASK: u64 = (1 << 48) - 1;
const XOR_SEED: u64 = 0x5DEECE66D;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct FastRandom {
    seed: u64,
}

impl FastRandom {
    #[inline(always)]
    pub fn new() -> Self {
        FastRandom { seed: 0 }
    }

    #[inline(always)]
    pub fn set_seed(&mut self, seed: i64) {
        self.seed = ((seed as u64) ^ XOR_SEED) & LCG_MASK;
    }

    #[inline(always)]
    pub fn next31(&mut self) -> u32 {
        self.seed = self.seed
            .wrapping_mul(LCG_MULT)
            .wrapping_add(LCG_ADD)
            & LCG_MASK;
        (self.seed >> 17) as u32
    }

    #[inline(always)]
    pub fn next_int_mod(&mut self, bound: i32) -> i32 {
        let bits = self.next31() as i64;
        (bits % bound as i64) as i32
    }
}

#[inline(always)]
pub fn compute_fx(x: i32) -> i64 {
    let xi = x as i64;
    xi.wrapping_mul(xi) * C_X2 + xi * C_X1
}

#[inline(always)]
pub fn compute_fz(z: i32) -> i64 {
    let zi = z as i64;
    zi.wrapping_mul(zi) * C_Z2 + zi * C_Z1
}

#[inline(always)]
pub fn is_slime_chunk_fast(rng: &mut FastRandom, fx: i64, fz: i64, seed: i64) -> bool {
    let combined = (seed as u64).wrapping_add(fx as u64).wrapping_add(fz as u64) as i64;
    rng.set_seed(combined ^ XOR_MASK);
    rng.next_int_mod(BOUND) == TARGET
}

#[inline(always)]
pub fn is_slime_chunk(x: i32, z: i32, seed: i64) -> bool {
    let mut rng = FastRandom::new();
    let fx = compute_fx(x);
    let fz = compute_fz(z);
    is_slime_chunk_fast(&mut rng, fx, fz, seed)
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
        assert_eq!(is_slime_chunk(0, 0, seed), is_slime_chunk(0, 0, seed));
    }

    #[test]
    fn test_is_slime_chunk_fast_equivalence() {
        let seed = 20260627i64;
        let fx = compute_fx(100);
        let fz = compute_fz(200);
        let mut rng = FastRandom::new();
        assert_eq!(is_slime_chunk_fast(&mut rng, fx, fz, seed), is_slime_chunk(100, 200, seed));
    }
}
