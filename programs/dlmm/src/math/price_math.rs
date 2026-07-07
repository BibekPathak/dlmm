use anchor_lang::prelude::*;
use crate::errors::DlmmError;

use super::fixed_point::{base_multiplier, inv_base_multiplier, q64_mul, Q64};

pub const BIN_ID_RANGE: i32 = 100_000;

pub fn bin_to_price(bin_id: i32, step_bps: u16) -> Result<u128> {
    if bin_id >= 0 {
        let base = base_multiplier(step_bps);
        pow_q64(base, bin_id as u64)
    } else {
        let inv_base = inv_base_multiplier(step_bps);
        pow_q64(inv_base, (-bin_id) as u64)
    }
}

pub fn pow_q64(base: u128, exp: u64) -> Result<u128> {
    if exp == 0 {
        return Ok(Q64);
    }
    let mut result = Q64;
    let mut b = base;
    let mut e = exp;
    while e > 0 {
        if e & 1 == 1 {
            result = q64_mul(result, b)?;
        }
        b = q64_mul(b, b)?;
        e >>= 1;
    }
    Ok(result)
}

pub fn price_to_bin(price_q64: u128, step_bps: u16) -> Result<i32> {
    let mut lo = -BIN_ID_RANGE;
    let mut hi = BIN_ID_RANGE;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let mid_price = bin_to_price(mid, step_bps)?;
        if mid_price < price_q64 {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    Ok(lo)
}

pub fn next_bin_price(bin_id: i32, step_bps: u16) -> Result<u128> {
    bin_to_price(bin_id.saturating_add(1), step_bps)
}

pub fn prev_bin_price(bin_id: i32, step_bps: u16) -> Result<u128> {
    bin_to_price(bin_id.saturating_sub(1), step_bps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::fixed_point::q64_div;

    const STEP_100: u16 = 100;
    const STEP_1: u16 = 1;
    const STEP_0: u16 = 0;

    #[test]
    fn test_pow_q64_exp_zero() {
        assert_eq!(pow_q64(base_multiplier(100), 0).unwrap(), Q64);
    }

    #[test]
    fn test_pow_q64_exp_one() {
        let base = base_multiplier(100);
        assert_eq!(pow_q64(base, 1).unwrap(), base);
    }

    #[test]
    fn test_pow_q64_exp_two() {
        let base = base_multiplier(100);
        let squared = q64_mul(base, base).unwrap();
        assert_eq!(pow_q64(base, 2).unwrap(), squared);
    }

    #[test]
    fn test_pow_q64_exp_ten() {
        let base = base_multiplier(100);
        let mut manual = Q64;
        for _ in 0..10 {
            manual = q64_mul(manual, base).unwrap();
        }
        assert_eq!(pow_q64(base, 10).unwrap(), manual);
    }

    #[test]
    fn test_pow_q64_binary_vs_iterative() {
        let base = base_multiplier(1);
        let exp = 50u64;
        let mut iterative = Q64;
        for _ in 0..exp {
            iterative = q64_mul(iterative, base).unwrap();
        }
        let binary = pow_q64(base, exp).unwrap();
        let diff = if binary > iterative { binary - iterative } else { iterative - binary };
        assert!(diff <= 20, "binary vs iterative diff={}", diff);
    }

    #[test]
    fn test_pow_q64_large_exp() {
        let base = base_multiplier(1);
        let result = pow_q64(base, 100_000).unwrap();
        assert!(result > Q64);
        assert!(result < u128::MAX);
    }

    #[test]
    fn test_pow_q64_overflow() {
        let base = base_multiplier(10_000);
        assert!(pow_q64(base, 1000).is_err());
    }

    #[test]
    fn test_bin_to_price_bin_zero() {
        assert_eq!(bin_to_price(0, STEP_100).unwrap(), Q64);
    }

    #[test]
    fn test_bin_to_price_bin_one() {
        let base = base_multiplier(STEP_100);
        assert_eq!(bin_to_price(1, STEP_100).unwrap(), base);
    }

    #[test]
    fn test_bin_to_price_bin_neg_one() {
        let inv = inv_base_multiplier(STEP_100);
        assert_eq!(bin_to_price(-1, STEP_100).unwrap(), inv);
    }

    #[test]
    fn test_bin_to_price_neg_vs_pos() {
        let p1 = bin_to_price(5, STEP_100).unwrap();
        let p2 = bin_to_price(-5, STEP_100).unwrap();
        let product = q64_mul(p1, p2).unwrap();
        let diff = if product > Q64 { product - Q64 } else { Q64 - product };
        assert!(diff <= 10, "p1*neg should be ~1.0, diff={}", diff);
    }

    #[test]
    fn test_bin_to_price_zero_step() {
        assert_eq!(bin_to_price(5000, STEP_0).unwrap(), Q64);
        assert_eq!(bin_to_price(-5000, STEP_0).unwrap(), Q64);
    }

    #[test]
    fn test_bin_to_price_monotonic() {
        let prev = bin_to_price(-10, STEP_100).unwrap();
        for i in -9..=10i32 {
            let cur = bin_to_price(i, STEP_100).unwrap();
            assert!(cur > prev, "bin {} price should be > bin {}", i, i - 1);
        }
    }

    #[test]
    fn test_bin_to_price_max_range() {
        let min_price = bin_to_price(-BIN_ID_RANGE, STEP_1).unwrap();
        let max_price = bin_to_price(BIN_ID_RANGE, STEP_1).unwrap();
        assert!(min_price < Q64);
        assert!(max_price > Q64);
        assert!(min_price > 0);
        assert!(max_price < u128::MAX);
    }

    #[test]
    fn test_bin_to_price_overflow_positive() {
        assert!(bin_to_price(BIN_ID_RANGE, 10_000).is_err());
    }

    #[test]
    fn test_price_to_bin_identity_bin_zero() {
        let bin = price_to_bin(Q64, STEP_100).unwrap();
        assert_eq!(bin, 0);
    }

    #[test]
    fn test_price_to_bin_roundtrip() {
        for &bin_id in &[0, 1, 5, 100, -1, -5, -100] {
            let price = bin_to_price(bin_id, STEP_1).unwrap();
            let recovered = price_to_bin(price, STEP_1).unwrap();
            assert_eq!(bin_id, recovered, "roundtrip failed for bin {}", bin_id);
        }
    }

    #[test]
    fn test_price_to_bin_roundtrip_large() {
        for &bin_id in &[1000, -1000, 50000, -50000] {
            let price = bin_to_price(bin_id, STEP_1).unwrap();
            let recovered = price_to_bin(price, STEP_1).unwrap();
            assert_eq!(bin_id, recovered, "roundtrip failed for bin {}", bin_id);
        }
    }

    #[test]
    fn test_next_prev_bin_price() {
        let base = base_multiplier(100);
        let inv = inv_base_multiplier(100);
        let b1 = bin_to_price(5, STEP_100).unwrap();
        let next = next_bin_price(5, STEP_100).unwrap();
        let prev = prev_bin_price(5, STEP_100).unwrap();
        assert_eq!(next, q64_mul(b1, base).unwrap());
        let prev_expected = q64_mul(b1, inv).unwrap();
        let diff = if prev > prev_expected { prev - prev_expected } else { prev_expected - prev };
        assert!(diff <= 5, "prev price mismatch diff={}", diff);
    }

    #[test]
    fn test_price_1pct_bins() {
        let base = base_multiplier(100);
        let p0 = bin_to_price(0, 100).unwrap();
        let p100 = bin_to_price(100, 100).unwrap();
        let expected = pow_q64(base, 100).unwrap();
        assert_eq!(p100, expected);
        let p100_by_inv = bin_to_price(-100, 100).unwrap();
        let product = q64_mul(p100, p100_by_inv).unwrap();
        let diff = if product > Q64 { product - Q64 } else { Q64 - product };
        assert!(diff <= 200, "p100 * p-100 should be ~Q64, diff={}", diff);
    }
}
