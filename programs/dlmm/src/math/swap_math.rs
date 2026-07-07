use anchor_lang::prelude::*;
use crate::errors::DlmmError;

pub struct SwapStepResult {
    pub amount_in_consumed: u64,
    pub amount_out: u64,
    pub fee_paid: u64,
    pub bin_depleted: bool,
}

pub fn compute_swap_step(
    _amount_in_remaining: u64,
    _bin_amount_x: u64,
    _bin_amount_y: u64,
    _bin_price_q64: u128,
    _a_to_b: bool,
    _fee_bps: u16,
) -> Result<SwapStepResult> {
    Err(error!(DlmmError::NotImplemented))
}
