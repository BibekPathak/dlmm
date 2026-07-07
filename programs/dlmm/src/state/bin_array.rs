use anchor_lang::prelude::*;

pub const BINS_PER_ARRAY: usize = 64;

#[zero_copy]
#[repr(C)]
pub struct Bin {
    pub amount_x: u64,
    pub amount_y: u64,
    pub fee_x: u64,
    pub fee_y: u64,
}

#[account(zero_copy)]
#[repr(C)]
pub struct BinArray {
    pub pool: Pubkey,
    pub start_bin_id: i32,
    pub bump: u8,
    pub reserved: [u8; 3],
    pub bins: [Bin; BINS_PER_ARRAY],
}

impl BinArray {
    pub const LEN: usize = 8
        + 32 // pool
        + 4 // start bin id
        + 1 // bump
        + 3 // reserved padding
        + (32 * BINS_PER_ARRAY); // bins
}
