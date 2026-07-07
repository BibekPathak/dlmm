use anchor_lang::prelude::*;
use super::fixed_point::BPS_DENOMINATOR;
use crate::errors::DlmmError;

pub fn apply_bps(amount: u64, bps: u16) -> Result<u64> {
    let result = (amount as u128)
        .checked_mul(bps as u128)
        .ok_or(error!(DlmmError::MathOverflow))?
        .checked_div(BPS_DENOMINATOR as u128)
        .ok_or(error!(DlmmError::DivisionByZero))?;
    Ok(result as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_bps_zero() {
        assert_eq!(apply_bps(1000, 0).unwrap(), 0);
    }

    #[test]
    fn test_apply_bps_100bps() {
        assert_eq!(apply_bps(10_000, 100).unwrap(), 100);
    }

    #[test]
    fn test_apply_bps_10000bps() {
        assert_eq!(apply_bps(500, 10_000).unwrap(), 500);
    }

    #[test]
    fn test_apply_bps_50bps() {
        assert_eq!(apply_bps(2000, 50).unwrap(), 10);
    }

    #[test]
    fn test_apply_bps_truncation() {
        assert_eq!(apply_bps(100, 1).unwrap(), 0);
        assert_eq!(apply_bps(100, 99).unwrap(), 0);
        assert_eq!(apply_bps(100, 100).unwrap(), 1);
    }

    #[test]
    fn test_apply_bps_zero_amount() {
        assert_eq!(apply_bps(0, 500).unwrap(), 0);
    }

    #[test]
    fn test_apply_bps_large_amount() {
        assert_eq!(apply_bps(u64::MAX, 10_000).unwrap(), u64::MAX);
    }
}
