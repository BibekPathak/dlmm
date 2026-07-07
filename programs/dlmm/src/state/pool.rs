use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub authority: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_vault_a: Pubkey,
    pub token_vault_b: Pubkey,
    pub fee_tier_bps: u16,
    pub protocol_fee_bps: u16,
    pub bin_step_bps: u16,
    pub base_bin_id: i32,
    pub active_bin_id: i32,
    pub pending_protocol_fees_x: u64,
    pub pending_protocol_fees_y: u64,
    pub base_fee_bps: u16,
    pub variable_fee_bps: u16,
    pub vol_reference_price: u128,
    pub vol_accumulator: u64,
    pub vol_last_timestamp: i64,
    pub fee_decay_interval: u64,
    pub bump: u8,
    pub reserved: [u8; 7],
}

impl Pool {
    pub const SPACE: usize = 8
        + 32 // authority
        + 32 // mint a
        + 32 // mint b
        + 32 // vault a
        + 32 // vault b
        + 2 // fee tier bps
        + 2 // protocol fee bps
        + 2 // bin step bps
        + 4 // base bin id
        + 4 // active bin id
        + 8 // pending fees x
        + 8 // pending fees y
        + 2 // base fee bps
        + 2 // variable fee bps
        + 16 // vol reference price
        + 8 // vol accumulator
        + 8 // vol last timestamp
        + 8 // fee decay interval
        + 1 // bump
        + 7; // reserved
}
