use anchor_lang::prelude::*;
use crate::errors::DlmmError;
use super::fixed_point::Q64;

pub fn x_to_y(amount_x: u64, price_q64: u128) -> Result<u64> {
    let product = (amount_x as u128)
        .checked_mul(price_q64)
        .ok_or(error!(DlmmError::MathOverflow))?;
    Ok((product >> 64) as u64)
}

pub fn y_to_x(amount_y: u64, price_q64: u128) -> Result<u64> {
    let scaled = (amount_y as u128)
        .checked_mul(Q64)
        .ok_or(error!(DlmmError::MathOverflow))?;
    let result = scaled
        .checked_div(price_q64)
        .ok_or(error!(DlmmError::DivisionByZero))?;
    Ok(result as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x_to_y_price_one() {
        let amount = 1000u64;
        let price = Q64;
        assert_eq!(x_to_y(amount, price).unwrap(), amount);
    }

    #[test]
    fn test_x_to_y_price_two() {
        let amount = 1000u64;
        let price = Q64.saturating_mul(2);
        assert_eq!(x_to_y(amount, price).unwrap(), 2000);
    }

    #[test]
    fn test_x_to_y_price_half() {
        let amount = 1000u64;
        let price = Q64 / 2;
        assert_eq!(x_to_y(amount, price).unwrap(), 500);
    }

    #[test]
    fn test_x_to_y_zero() {
        assert_eq!(x_to_y(0, Q64).unwrap(), 0);
    }

    #[test]
    fn test_y_to_x_price_one() {
        let amount = 1000u64;
        let price = Q64;
        assert_eq!(y_to_x(amount, price).unwrap(), amount);
    }

    #[test]
    fn test_y_to_x_price_two() {
        let amount = 2000u64;
        let price = Q64.saturating_mul(2);
        assert_eq!(y_to_x(amount, price).unwrap(), 1000);
    }

    #[test]
    fn test_y_to_x_price_half() {
        let amount = 500u64;
        let price = Q64 / 2;
        assert_eq!(y_to_x(amount, price).unwrap(), 1000);
    }

    #[test]
    fn test_y_to_x_zero() {
        assert_eq!(y_to_x(0, Q64).unwrap(), 0);
    }

    #[test]
    fn test_y_to_x_div_by_zero() {
        assert!(y_to_x(100, 0).is_err());
    }

    #[test]
    fn test_x_to_y_roundtrip() {
        let x = 42_000u64;
        let price = Q64.saturating_mul(3) / 2;
        let y = x_to_y(x, price).unwrap();
        let x_back = y_to_x(y, price).unwrap();
        assert_eq!(x, x_back);
    }
}
