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
    pub bump: u8,
}

impl Pool {
    pub const SPACE: usize = 8
        + 32 // authority
        + 32 // mint a
        + 32 // mint b
        + 32 // vault a
        + 32 // vault b
        + 2 // fee bps
        + 2 // protocol fee bps
        + 2 // bin step bps
        + 4 // base bin id
        + 1; // bump
}

#[account]
pub struct Position {
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub lower_bin_id: i32,
    pub upper_bin_id: i32,
    pub liquidity: u128,
    pub fees_owed_a: u64,
    pub fees_owed_b: u64,
    pub bump: u8,
}

impl Position {
    pub const SPACE: usize = 8
        + 32 // owner
        + 32 // pool
        + 4 // lower bin
        + 4 // upper bin
        + 16 // liquidity
        + 8 // fees a
        + 8 // fees b
        + 1; // bump
}

#[account]
pub struct FeeTier {
    pub lp_fee_bps: u16,
    pub protocol_fee_bps: u16,
}


