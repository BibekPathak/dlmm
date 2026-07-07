use anchor_lang::prelude::*;

#[account]
pub struct Position {
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub lower_bin_id: i32,
    pub upper_bin_id: i32,
    pub total_liquidity_x: u64,
    pub total_liquidity_y: u64,
    pub fee_checkpoint_x: u64,
    pub fee_checkpoint_y: u64,
    pub fees_owed_x: u64,
    pub fees_owed_y: u64,
    pub last_update: i64,
    pub bump: u8,
}

impl Position {
    pub const SPACE: usize = 8
        + 32 // owner
        + 32 // pool
        + 4 // lower bin
        + 4 // upper bin
        + 8 // total liq x
        + 8 // total liq y
        + 8 // fee checkpoint x
        + 8 // fee checkpoint y
        + 8 // fees owed x
        + 8 // fees owed y
        + 8 // last update
        + 1; // bump

    pub fn is_in_range(&self, bin_id: i32) -> bool {
        bin_id >= self.lower_bin_id && bin_id <= self.upper_bin_id
    }
}
