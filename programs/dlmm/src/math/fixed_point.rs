use anchor_lang::prelude::*;
use crate::errors::DlmmError;

pub const Q64: u128 = 1u128 << 64;
pub const BPS_DENOMINATOR: u128 = 10_000;

pub fn q64_mul(a: u128, b: u128) -> Result<u128> {
    let mul = a.checked_mul(b).ok_or(error!(DlmmError::MathOverflow))?;
    mul.checked_div(Q64).ok_or(error!(DlmmError::MathOverflow))
}

pub fn q64_div(a: u128, b: u128) -> Result<u128> {
    let scaled = a.checked_mul(Q64).ok_or(error!(DlmmError::MathOverflow))?;
    scaled.checked_div(b).ok_or(error!(DlmmError::DivisionByZero))
}

pub fn base_multiplier(step_bps: u16) -> u128 {
    Q64 + (step_bps as u128 * Q64 / BPS_DENOMINATOR)
}
