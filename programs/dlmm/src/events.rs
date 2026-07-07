use anchor_lang::prelude::*;

#[event]
pub struct SwapEvent {
    pub pool: Pubkey,
    pub payer: Pubkey,
    pub a_to_b: bool,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee: u64,
    pub active_bin_after: i32,
    pub bins_traversed: u64,
}

#[event]
pub struct LiquidityEvent {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub bin_ids: Vec<i32>,
    pub amounts_x: Vec<u64>,
    pub amounts_y: Vec<u64>,
    pub is_deposit: bool,
}
