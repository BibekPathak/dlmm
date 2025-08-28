#![allow(unused)]

pub const BPS_DENOMINATOR: u64 = 10_000;

pub fn apply_bps(amount: u64, bps: u16) -> u64 {
    (amount as u128 * bps as u128 / BPS_DENOMINATOR as u128) as u64
}

pub fn bin_id_to_price_x64(_base_bin_id: i32, _bin_step_bps: u16) -> u128 {
    // Placeholder: return 1.0 in Q32.32 format to avoid floating point on BPF
    1u128 << 32
}


