use anchor_lang::prelude::*;
use crate::errors::DlmmError;

pub const Q64: u128 = 1u128 << 64;
pub const BPS_DENOMINATOR: u128 = 10_000;

pub fn q64_mul(a: u128, b: u128) -> Result<u128> {
    let a_lo = a as u64 as u128;
    let a_hi = a >> 64;
    let b_lo = b as u64 as u128;
    let b_hi = b >> 64;

    let ab_hi_hi = a_hi.checked_mul(b_hi).ok_or(error!(DlmmError::MathOverflow))?;
    if ab_hi_hi > u64::MAX as u128 {
        return Err(error!(DlmmError::MathOverflow));
    }
    let term1 = ab_hi_hi << 64;

    let term2 = a_hi.checked_mul(b_lo).ok_or(error!(DlmmError::MathOverflow))?;
    let term3 = a_lo.checked_mul(b_hi).ok_or(error!(DlmmError::MathOverflow))?;
    let term4 = (a_lo * b_lo) >> 64;

    let result = term1
        .checked_add(term2).ok_or(error!(DlmmError::MathOverflow))?
        .checked_add(term3).ok_or(error!(DlmmError::MathOverflow))?
        .checked_add(term4).ok_or(error!(DlmmError::MathOverflow))?;

    Ok(result)
}

pub fn q64_div(a: u128, b: u128) -> Result<u128> {
    if b == 0 {
        return Err(error!(DlmmError::DivisionByZero));
    }
    let scaled = a.checked_mul(Q64).ok_or(error!(DlmmError::MathOverflow))?;
    scaled.checked_div(b).ok_or(error!(DlmmError::DivisionByZero))
}

pub fn base_multiplier(step_bps: u16) -> u128 {
    Q64 + (step_bps as u128 * Q64 / BPS_DENOMINATOR)
}

pub fn inv_base_multiplier(step_bps: u16) -> u128 {
    Q64 * BPS_DENOMINATOR / (BPS_DENOMINATOR + step_bps as u128)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_q64_mul_identity() {
        assert_eq!(q64_mul(Q64, Q64).unwrap(), Q64);
    }

    #[test]
    fn test_q64_mul_scalar() {
        let two = Q64.saturating_mul(2);
        let three = Q64.saturating_mul(3);
        let six = Q64.saturating_mul(6);
        assert_eq!(q64_mul(two, three).unwrap(), six);
    }

    #[test]
    fn test_q64_mul_large_values() {
        let a = 100_000u128.saturating_mul(Q64);
        let b = Q64;
        assert_eq!(q64_mul(a, b).unwrap(), a);
    }

    #[test]
    fn test_q64_mul_fraction() {
        let half = Q64 / 2;
        let quarter = Q64 / 4;
        let result = q64_mul(half, half).unwrap();
        let diff = if result > quarter { result - quarter } else { quarter - result };
        assert!(diff <= 1, "quarter={} result={} diff={}", quarter, result, diff);
    }

    #[test]
    fn test_q64_mul_overflow() {
        assert!(q64_mul(u128::MAX, u128::MAX).is_err());
        assert!(q64_mul(u128::MAX, Q64.saturating_mul(2)).is_err());
    }

    #[test]
    fn test_q64_mul_zero() {
        assert_eq!(q64_mul(0, Q64).unwrap(), 0);
        assert_eq!(q64_mul(Q64, 0).unwrap(), 0);
    }

    #[test]
    fn test_q64_div_identity() {
        let small = Q64 / 2;
        assert_eq!(q64_div(small, small).unwrap(), Q64);
    }

    #[test]
    fn test_q64_div_small_a() {
        let a = 5000u128;
        let b = Q64;
        let result = q64_div(a, b).unwrap();
        assert_eq!(result, 5000);
    }

    #[test]
    fn test_q64_div_half() {
        let half: u128 = Q64 / 2;
        let result = q64_div(half, Q64).unwrap();
        assert_eq!(result, Q64 / 2);
    }

    #[test]
    fn test_q64_div_by_zero() {
        let small = Q64 / 2;
        assert!(q64_div(small, 0).is_err());
    }

    #[test]
    fn test_q64_div_zero_numerator() {
        assert_eq!(q64_div(0, Q64).unwrap(), 0);
    }

    #[test]
    fn test_inv_base_multiplier_zero() {
        assert_eq!(inv_base_multiplier(0), Q64);
    }

    #[test]
    fn test_inv_base_multiplier_100bps() {
        let inv = inv_base_multiplier(100);
        let base = base_multiplier(100);
        let product = q64_mul(base, inv).unwrap();
        let diff = if product > Q64 { product - Q64 } else { Q64 - product };
        assert!(diff <= 1, "base * inv should be ~Q64, diff={}", diff);
    }

    #[test]
    fn test_inv_base_vs_reciprocal() {
        for step in [1, 10, 50, 100, 500, 1000, 5000] {
            let base = base_multiplier(step);
            let inv = inv_base_multiplier(step);
            let product = q64_mul(base, inv).unwrap();
            let diff = if product > Q64 { product - Q64 } else { Q64 - product };
            assert!(diff <= 2, "step={}: base*inv diff={}", step, diff);
        }
    }

    #[test]
    fn test_base_multiplier_zero_step() {
        assert_eq!(base_multiplier(0), Q64);
    }

    #[test]
    fn test_base_multiplier_100bps() {
        let base = base_multiplier(100);
        let step_q64 = Q64 / 100;
        assert_eq!(base, Q64 + step_q64);
    }

    #[test]
    fn test_base_multiplier_10000bps() {
        let base = base_multiplier(10000);
        assert_eq!(base, Q64.saturating_mul(2));
    }

    #[test]
    fn test_q64_mul_max_valid() {
        let max_valid = u64::MAX as u128;
        assert!(q64_mul(max_valid, Q64).is_ok());
    }

    #[test]
    fn test_q64_mul_many_products() {
        let vals = [Q64 / 2, Q64, Q64 * 3 / 2, Q64 * 2, Q64 * 10];
        for &a in &vals {
            assert!(q64_mul(a, Q64).is_ok());
        }
    }

    #[test]
    fn test_q64_div_by_one() {
        let a = 12345u128;
        assert_eq!(q64_div(a, Q64).unwrap(), a);
    }

    #[test]
    fn test_q64_mul_commutative() {
        let a = Q64 * 3 / 2;
        let b = Q64 * 7 / 4;
        let ab = q64_mul(a, b).unwrap();
        let ba = q64_mul(b, a).unwrap();
        assert_eq!(ab, ba);
    }

    #[test]
    fn test_inv_base_multiplier_max_step() {
        let inv = inv_base_multiplier(10000);
        assert_eq!(inv, Q64 / 2);
    }
}
