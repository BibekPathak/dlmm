use anchor_lang::prelude::*;
use crate::errors::DlmmError;
use super::fee_math::apply_bps;
use super::fixed_point::Q64;

pub struct SwapStepResult {
    pub amount_in_consumed: u64,
    pub amount_out: u64,
    pub fee_paid: u64,
    pub bin_depleted: bool,
}

pub fn compute_swap_step(
    remaining_net: u64,
    bin_amount_x: u64,
    bin_amount_y: u64,
    bin_price_q64: u128,
    a_to_b: bool,
    fee_bps: u16,
) -> Result<SwapStepResult> {
    if a_to_b {
        let available = bin_amount_y;
        if available == 0 {
            return Ok(SwapStepResult {
                amount_in_consumed: 0,
                amount_out: 0,
                fee_paid: 0,
                bin_depleted: false,
            });
        }

        let desired_out = ((remaining_net as u128).checked_mul(bin_price_q64)
            .ok_or(error!(DlmmError::MathOverflow))? >> 64) as u64;

        if desired_out <= available {
            Ok(SwapStepResult {
                amount_in_consumed: remaining_net,
                amount_out: desired_out,
                fee_paid: apply_bps(remaining_net, fee_bps)?,
                bin_depleted: false,
            })
        } else {
            let net_needed = ((available as u128)
                .checked_mul(Q64)
                .ok_or(error!(DlmmError::MathOverflow))?
                .checked_div(bin_price_q64)
                .ok_or(error!(DlmmError::DivisionByZero))?) as u64;

            Ok(SwapStepResult {
                amount_in_consumed: net_needed,
                amount_out: available,
                fee_paid: apply_bps(net_needed, fee_bps)?,
                bin_depleted: true,
            })
        }
    } else {
        let available = bin_amount_x;
        if available == 0 {
            return Ok(SwapStepResult {
                amount_in_consumed: 0,
                amount_out: 0,
                fee_paid: 0,
                bin_depleted: false,
            });
        }

        let desired_out = ((remaining_net as u128)
            .checked_mul(Q64)
            .ok_or(error!(DlmmError::MathOverflow))?
            .checked_div(bin_price_q64)
            .ok_or(error!(DlmmError::DivisionByZero))?) as u64;

        if desired_out <= available {
            Ok(SwapStepResult {
                amount_in_consumed: remaining_net,
                amount_out: desired_out,
                fee_paid: apply_bps(remaining_net, fee_bps)?,
                bin_depleted: false,
            })
        } else {
            let net_needed = ((available as u128)
                .checked_mul(bin_price_q64)
                .ok_or(error!(DlmmError::MathOverflow))? >> 64) as u64;

            Ok(SwapStepResult {
                amount_in_consumed: net_needed,
                amount_out: available,
                fee_paid: apply_bps(net_needed, fee_bps)?,
                bin_depleted: true,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const Q64: u128 = 1u128 << 64;

    #[test]
    fn test_step_empty_bin() {
        let r = compute_swap_step(1000, 100, 0, Q64, true, 0).unwrap();
        assert_eq!(r.amount_out, 0);
        assert!(!r.bin_depleted);
    }

    #[test]
    fn test_step_partial_fill_a_to_b() {
        let r = compute_swap_step(100, 500, 500, Q64, true, 0).unwrap();
        assert_eq!(r.amount_out, 100);
        assert_eq!(r.amount_in_consumed, 100);
        assert!(!r.bin_depleted);
    }

    #[test]
    fn test_step_full_deplete_a_to_b() {
        let r = compute_swap_step(1000, 500, 50, Q64, true, 0).unwrap();
        assert_eq!(r.amount_out, 50);
        assert_eq!(r.amount_in_consumed, 50);
        assert!(r.bin_depleted);
    }

    #[test]
    fn test_step_partial_fill_b_to_a() {
        let r = compute_swap_step(100, 500, 500, Q64, false, 0).unwrap();
        assert_eq!(r.amount_out, 100);
        assert!(!r.bin_depleted);
    }

    #[test]
    fn test_step_with_fee() {
        let r = compute_swap_step(1000, 5000, 5000, Q64, true, 100).unwrap();
        assert_eq!(r.fee_paid, 10);
        assert_eq!(r.amount_out, 1000);
    }
}
