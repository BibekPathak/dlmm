use anchor_lang::prelude::*;
use crate::state::bin_array::bin_id_to_array_start;
use crate::state::bin_array::BINS_PER_ARRAY;
use crate::math::price_math::BIN_ID_RANGE;

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

    pub fn bin_array_seed(bin_id: i32) -> Vec<u8> {
        let start = bin_id_to_array_start(bin_id);
        start.to_le_bytes().to_vec()
    }

    pub fn derive_bin_array_address(pool_key: &Pubkey, bin_id: i32, program_id: &Pubkey) -> (Pubkey, u8) {
        let start = bin_id_to_array_start(bin_id);
        Pubkey::find_program_address(
            &[
                b"bin_array",
                pool_key.as_ref(),
                &start.to_le_bytes(),
            ],
            program_id,
        )
    }

    pub fn active_bin_array_start(&self) -> i32 {
        bin_id_to_array_start(self.active_bin_id)
    }
}

#[cfg(test)]
impl Pool {
    pub fn check_invariants(&self) {
        // P1: active_bin_id in range
        assert!(self.active_bin_id >= -BIN_ID_RANGE && self.active_bin_id <= BIN_ID_RANGE,
            "P1: active_bin_id={} out of range", self.active_bin_id);
        // P2: bin_step_bps valid
        assert!(self.bin_step_bps > 0 && self.bin_step_bps <= 10000,
            "P2: bin_step_bps={} invalid", self.bin_step_bps);
        // P3: base_fee_bps not excessive
        assert!(self.base_fee_bps <= 10000,
            "P3: base_fee_bps={} > 10000", self.base_fee_bps);
        // P4: protocol_fee_bps ≤ base_fee_bps
        assert!(self.protocol_fee_bps <= self.base_fee_bps,
            "P4: protocol_fee_bps={} > base_fee_bps={}", self.protocol_fee_bps, self.base_fee_bps);
        // P5: pending fees non-negative (u64, automatically)
        // P6: variable_fee_bps bounded
        assert!(self.variable_fee_bps <= 200,
            "P6: variable_fee_bps={} > 200", self.variable_fee_bps);
    }
}
