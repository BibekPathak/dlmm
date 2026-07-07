use anchor_lang::prelude::*;

#[account]
pub struct FeeTier {
    pub lp_fee_bps: u16,
    pub protocol_fee_bps: u16,
}

impl FeeTier {
    pub const SPACE: usize = 8 + 2 + 2;
}
