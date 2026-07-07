use anchor_lang::prelude::*;
use super::fixed_point::BPS_DENOMINATOR;
use crate::errors::DlmmError;

pub fn apply_bps(amount: u64, bps: u16) -> Result<u64> {
    let result = (amount as u128)
        .checked_mul(bps as u128)
        .ok_or(error!(DlmmError::MathOverflow))?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(error!(DlmmError::DivisionByZero))?;
    Ok(result as u64)
}
