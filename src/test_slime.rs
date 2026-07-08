use crate::slime_chunk::{compute_fx, compute_fz, is_slime_chunk_fast};

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
        (999999, 999999, false),
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
