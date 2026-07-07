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
