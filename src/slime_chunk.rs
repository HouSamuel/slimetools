const C_X2: i32 = 4987142;
const C_X1: i32 = 5947611;
const C_Z2: i32 = 4392871;
const C_Z1: i32 = 389711;
const XOR_MASK: i64 = 987234911;
const BOUND: i32 = 10;
const TARGET: i32 = 0;

const LCG_MULT: i64 = 0x5DEECE66D;
const LCG_ADD: i64 = 0xB;
const LCG_MASK: i64 = (1 << 48) - 1;

#[inline(always)]
pub fn compute_fx(x: i32) -> i64 {
    let xi = x;
    let term1 = ((xi.wrapping_mul(xi)) as i64).wrapping_mul(C_X2 as i64);
    let term2 = (xi.wrapping_mul(C_X1)) as i64;
    term1 + term2
}

#[inline(always)]
pub fn compute_fz(z: i32) -> i64 {
    let zi = z;
    let term3 = ((zi.wrapping_mul(zi)) as i64).wrapping_mul(C_Z2 as i64);
    let term4 = (zi.wrapping_mul(C_Z1)) as i64;
    term3 + term4
}

#[inline(always)]
pub fn get_random_seed(world_seed: i64, x: i32, z: i32) -> i64 {
    let x_sq = (x.wrapping_mul(x).wrapping_mul(C_X2)) as i64;
    let x_lin = (x.wrapping_mul(C_X1)) as i64;
    let z_sq = (z.wrapping_mul(z).wrapping_mul(C_Z2)) as i64;
    let z_lin = (z.wrapping_mul(C_Z1)) as i64;
    world_seed.wrapping_add(x_sq).wrapping_add(x_lin).wrapping_add(z_sq).wrapping_add(z_lin) ^ XOR_MASK
}

#[inline(always)]
fn next(seed: &mut i64, bits: i32) -> i32 {
    *seed = (*seed * LCG_MULT + LCG_ADD) & LCG_MASK;
    ((*seed as u64) >> (48 - bits)) as i32
}

#[inline(always)]
fn next_int_biased(seed: &mut i64, bound: i32) -> i32 {
    let bits = next(seed, 31);
    bits % bound
}

#[inline(always)]
fn next_int(seed: &mut i64, bound: i32) -> i32 {
    let bits = next(seed, 31);
    let val = bits % bound;
    if bits - val + (bound - 1) >= 0 {
        return val;
    }
    loop {
        let bits = next(seed, 31);
        let val = bits % bound;
        if bits - val + (bound - 1) >= 0 {
            return val;
        }
    }
}

#[inline(always)]
pub fn is_slime_chunk_fast(fx: i64, fz: i64, world_seed: i64) -> bool {
    let seed = world_seed.wrapping_add(fx).wrapping_add(fz) ^ XOR_MASK;
    let mut lcg_seed = seed ^ LCG_MULT & LCG_MASK;
    lcg_seed = (lcg_seed * LCG_MULT + LCG_ADD) & LCG_MASK;
    let bits = ((lcg_seed as u64) >> 17) as i32;
    let val = bits % BOUND;
    val == TARGET
}

#[inline(always)]
pub fn is_slime_chunk(world_seed: i64, x: i32, z: i32) -> bool {
    let random_seed = get_random_seed(world_seed, x, z);
    let mut lcg_seed = random_seed ^ LCG_MULT & LCG_MASK;
    next_int(&mut lcg_seed, BOUND) == TARGET
}

#[inline(always)]
pub fn is_slime_chunk_scalar(world_seed: i64, x: i32, z: i32) -> bool {
    is_slime_chunk(world_seed, x, z)
}

#[inline(always)]
pub fn is_slime_chunk_biased(world_seed: i64, x: i32, z: i32) -> bool {
    let random_seed = get_random_seed(world_seed, x, z);
    let mut lcg_seed = random_seed ^ LCG_MULT & LCG_MASK;
    next_int_biased(&mut lcg_seed, BOUND) == TARGET
}

#[cfg(target_arch = "x86_64")]
pub mod simd {
    use std::arch::x86_64::*;
    
    pub const LANES: usize = 16;
    
    type Vec64 = __m512i;
    type Vec32 = __m512i;
    type Vecu8 = __m512i;
    
    #[inline(always)]
    unsafe fn splat_i64(v: i64) -> Vec64 {
        _mm512_set1_epi64(v)
    }
    
    #[inline(always)]
    unsafe fn splat_i32(v: i32) -> Vec32 {
        _mm512_set1_epi32(v)
    }
    
    #[inline(always)]
    unsafe fn add_epi64(a: Vec64, b: Vec64) -> Vec64 {
        _mm512_add_epi64(a, b)
    }
    
    #[inline(always)]
    unsafe fn xor_epi64(a: Vec64, b: Vec64) -> Vec64 {
        _mm512_xor_epi64(a, b)
    }
    
    #[inline(always)]
    unsafe fn mul_epi32(a: Vec32, b: Vec32) -> Vec32 {
        _mm512_mullo_epi32(a, b)
    }
    
    #[inline(always)]
    unsafe fn mul_epi64(a: Vec64, b: Vec64) -> Vec64 {
        _mm512_mul_epi64(a, b)
    }
    
    #[inline(always)]
    unsafe fn and_epi64(a: Vec64, b: Vec64) -> Vec64 {
        _mm512_and_epi64(a, b)
    }
    
    #[inline(always)]
    unsafe fn shr_epi64(a: Vec64, count: i32) -> Vec64 {
        _mm512_srli_epi64(a, count as u32)
    }
    
    #[inline(always)]
    unsafe fn cmp_eq_epi32(a: Vec32, b: Vec32) -> Vec32 {
        _mm512_cmpeq_epi32(a, b)
    }
    
    #[inline(always)]
    unsafe fn add_epi8(a: Vecu8, b: Vecu8) -> Vecu8 {
        _mm512_add_epi8(a, b)
    }
    
    #[inline(always)]
    unsafe fn shuffle_epi8(a: Vecu8, b: Vecu8, mask: Vecu8) -> Vecu8 {
        _mm512_shuffle_epi8(a, mask)
    }
    
    #[inline(always)]
    unsafe fn set_epi32_range(base: i32) -> Vec32 {
        _mm512_set_epi32(
            base + 15, base + 14, base + 13, base + 12,
            base + 11, base + 10, base + 9, base + 8,
            base + 7, base + 6, base + 5, base + 4,
            base + 3, base + 2, base + 1, base
        )
    }
    
    #[inline(always)]
    unsafe fn get_random_seeds(world_seed: i64, x: i32, z: i32) -> Vec64 {
        let world_seeds = splat_i64(world_seed);
        
        let x_sq = (x.wrapping_mul(x)).wrapping_mul(C_X2) as i64;
        let x_lin = (x.wrapping_mul(C_X1)) as i64;
        let x_increment = splat_i64(x_sq + x_lin);
        
        let magic1 = splat_i64(C_Z2 as i64);
        let magic2_i32 = splat_i32(C_Z1);
        
        let zs = set_epi32_range(z);
        let zs_sq = mul_epi32(zs, zs);
        let z_sq_part = mul_epi64(_mm512_cvtepi32_epi64(zs_sq), magic1);
        let z_lin_part = _mm512_cvtepi32_epi64(mul_epi32(zs, magic2_i32));
        
        let z_increment = add_epi64(z_sq_part, z_lin_part);
        let magic3 = splat_i64(XOR_MASK);
        
        xor_epi64(add_epi64(add_epi64(world_seeds, x_increment), z_increment), magic3)
    }
    
    struct RandomState {
        seed: Vec64,
    }
    
    #[inline(always)]
    unsafe fn random_init(seed: Vec64) -> RandomState {
        let multiplier = splat_i64(LCG_MULT);
        let mask = splat_i64(LCG_MASK);
        RandomState {
            seed: and_epi64(xor_epi64(seed, multiplier), mask),
        }
    }
    
    #[inline(always)]
    unsafe fn random_next(state: &mut RandomState, bits: i32) -> Vec32 {
        let multiplier = splat_i64(LCG_MULT);
        let addend = splat_i64(LCG_ADD);
        let mask = splat_i64(LCG_MASK);
        
        state.seed = and_epi64(add_epi64(mul_epi64(state.seed, multiplier), addend), mask);
        _mm512_cvtepi64_epi32(shr_epi64(state.seed, 48 - bits))
    }
    
    #[inline(always)]
    unsafe fn random_next_ints_biased(state: &mut RandomState, bound: i32) -> Vec32 {
        let bounds = splat_i32(bound);
        let bits = random_next(state, 31);
        _mm512_mod_epi32(bits, bounds)
    }
    
    #[inline(always)]
    pub unsafe fn are_slime_biased(world_seed: i64, x: i32, z: i32) -> Vecu8 {
        let seeds = get_random_seeds(world_seed, x, z);
        let mut random = random_init(seeds);
        let result = random_next_ints_biased(&mut random, BOUND);
        let zeros = splat_i32(0);
        let eq_mask = cmp_eq_epi32(result, zeros);
        
        let mut bytes: [u8; 64] = [0; 64];
        _mm512_storeu_si512(bytes.as_mut_ptr() as *mut __m512i, eq_mask);
        
        let mut result_bytes: [u8; 64] = [0; 64];
        for i in 0..LANES {
            result_bytes[i] = if bytes[i * 4] != 0 { 1 } else { 0 };
        }
        
        _mm512_loadu_si512(result_bytes.as_ptr() as *const __m512i)
    }
    
    #[inline(always)]
    pub fn scan_cell(v: Vecu8, carry: *mut u8) -> Vecu8 {
        unsafe {
            let zero = _mm512_setzero_si512();
            let mut result = v;
            
            const mask1: [i8; 64] = [-1; 64];
            let mask_arr1 = _mm512_loadu_si512(mask1.as_ptr() as *const __m512i);
            
            const mask2: [i8; 64] = [-1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7,
                                     -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7,
                                     -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7,
                                     -1, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7];
            let mask_arr2 = _mm512_loadu_si512(mask2.as_ptr() as *const __m512i);
            
            const mask3: [i8; 64] = [-1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
                                     -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
                                     -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
                                     -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
            let mask_arr3 = _mm512_loadu_si512(mask3.as_ptr() as *const __m512i);
            
            const mask4: [i8; 64] = [-1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
                                     -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
                                     -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
                                     -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
            let mask_arr4 = _mm512_loadu_si512(mask4.as_ptr() as *const __m512i);
            
            result = add_epi8(result, shuffle_epi8(result, zero, mask_arr1));
            result = add_epi8(result, shuffle_epi8(result, zero, mask_arr2));
            result = add_epi8(result, shuffle_epi8(result, zero, mask_arr3));
            result = add_epi8(result, shuffle_epi8(result, zero, mask_arr4));
            
            let carry_vec = _mm512_set1_epi8(*carry);
            result = add_epi8(result, carry_vec);
            
            let last_val = _mm512_extract_epi8(result, LANES - 1);
            *carry = last_val as u8;
            
            result
        }
    }
}
