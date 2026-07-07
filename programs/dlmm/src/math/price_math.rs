use anchor_lang::prelude::*;
use crate::errors::DlmmError;

use super::fixed_point::{base_multiplier, q64_div, q64_mul, Q64};

pub const BIN_ID_RANGE: i32 = 100_000;

pub fn bin_to_price(bin_id: i32, step_bps: u16) -> Result<u128> {
    let base = base_multiplier(step_bps);
    if bin_id >= 0 {
        pow_q64(base, bin_id as u64)
    } else {
        let positive = pow_q64(base, (-bin_id) as u64)?;
        q64_div(Q64, positive)
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
