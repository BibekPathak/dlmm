use anchor_lang::prelude::*;
use super::fixed_point::BPS_DENOMINATOR;
use crate::errors::DlmmError;

pub fn apply_bps(amount: u64, bps: u16) -> Result<u64> {
    let result = (amount as u128)
        .checked_mul(bps as u128)
        .ok_or(error!(DlmmError::MathOverflow))?
        .checked_div(BPS_DENOMINATOR as u128)
        .ok_or(error!(DlmmError::DivisionByZero))?;
    Ok(result as u64)
}

pub fn decay_volatility(
    accumulator: u64,
    last_timestamp: i64,
    current_timestamp: i64,
    decay_interval: u64,
) -> u64 {
    if current_timestamp <= last_timestamp || decay_interval == 0 {
        return accumulator;
    }
    let elapsed = (current_timestamp.saturating_sub(last_timestamp)) as u64;
    let intervals = elapsed / decay_interval;
    if intervals == 0 {
        return accumulator;
    }
    let mut decayed = accumulator;
    let capped = intervals.min(100);
    for _ in 0..capped {
        decayed = decayed.saturating_sub(decayed / 10);
    }
    decayed
}

pub fn calculate_variable_fee(vol_accumulator: u64, max_variable_fee_bps: u16) -> u16 {
    let vol_scaled = vol_accumulator.min(100_000) as u128;
    let max_var = max_variable_fee_bps as u128;
    (vol_scaled * max_var / 100_000) as u16
}

pub fn update_volatility(
    vol_accumulator: u64,
    vol_reference_price: u128,
    swap_price: u128,
    last_timestamp: i64,
    current_timestamp: i64,
    decay_interval: u64,
    max_variable_fee_bps: u16,
) -> (u64, u128, u16) {
    let decayed = decay_volatility(vol_accumulator, last_timestamp, current_timestamp, decay_interval);

    let diff = if swap_price > vol_reference_price {
        swap_price - vol_reference_price
    } else {
        vol_reference_price - swap_price
    };

    let vol_increment = if vol_reference_price > 0 {
        ((diff as u128) * 10_000 / vol_reference_price) as u64
    } else {
        0
    };

    let new_accumulator = decayed.saturating_add(vol_increment.min(50_000));
    let new_variable_fee = calculate_variable_fee(new_accumulator, max_variable_fee_bps);

    (new_accumulator, swap_price, new_variable_fee)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decay_volatility_no_elapsed() {
        assert_eq!(decay_volatility(1000, 100, 100, 60), 1000);
    }

    #[test]
    fn test_decay_volatility_zero_interval() {
        assert_eq!(decay_volatility(1000, 100, 200, 0), 1000);
    }

    #[test]
    fn test_decay_volatility_one_interval() {
        let d = decay_volatility(1000, 100, 160, 60);
        assert!(d < 1000 && d >= 900);
    }

    #[test]
    fn test_decay_volatility_many_intervals() {
        let d = decay_volatility(1000, 100, 100 + 60 * 10, 60);
        assert_eq!(d, 351);
    }

    #[test]
    fn test_calculate_variable_fee_zero() {
        assert_eq!(calculate_variable_fee(0, 200), 0);
    }

    #[test]
    fn test_calculate_variable_fee_maxed() {
        assert_eq!(calculate_variable_fee(100_000, 200), 200);
    }

    #[test]
    fn test_calculate_variable_fee_half() {
        assert_eq!(calculate_variable_fee(50_000, 200), 100);
    }

    #[test]
    fn test_update_volatility_basic() {
        let (acc, ref_price, var_fee) = update_volatility(
            0, 1u128 << 64, (11u128 << 64) / 10, 100, 200, 3600, 200,
        );
        // Integer truncation may give 999 instead of 1000
        assert!(acc == 999 || acc == 1000, "acc={}", acc);
        assert_eq!(ref_price, (11u128 << 64) / 10);
        assert!(var_fee == 1 || var_fee == 2, "var_fee={}", var_fee);
    }
}
