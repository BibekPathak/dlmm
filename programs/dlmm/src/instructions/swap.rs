use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use bytemuck::from_bytes_mut;
use crate::state::*;
use crate::errors::DlmmError;
use crate::events::SwapEvent;
use crate::math::price_math::bin_to_price;
use crate::math::swap_math::compute_swap_step;
use crate::math::fee_math::{apply_bps, update_volatility};
use crate::math::fixed_point::{q64_mul, Q64};
use crate::math::fixed_point::{base_multiplier, inv_base_multiplier};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SwapParams {
    pub amount: u64,
    pub a_to_b: bool,
    pub exact_in: bool,
    pub min_amount_out: u64,
    pub max_amount_in: u64,
    pub price_limit_bin_id: i32,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub token_vault_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_vault_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Swap>, params: SwapParams) -> Result<()> {
    require!(params.amount > 0, DlmmError::ZeroDeposit);
    if params.exact_in {
        require!(params.min_amount_out > 0, DlmmError::MinOutputNotMet);
    } else {
        require!(params.max_amount_in > 0, DlmmError::MaxInputExceeded);
    }

    let pool = &mut ctx.accounts.pool;
    let fee_bps = pool.base_fee_bps.saturating_add(pool.variable_fee_bps);
    let direction: i32 = if params.a_to_b { 1 } else { -1 };
    let mut cur_bin_id = pool.active_bin_id;
    let bin_step_bps = pool.bin_step_bps;
    let step_multiplier = if params.a_to_b {
        base_multiplier(bin_step_bps)
    } else {
        inv_base_multiplier(bin_step_bps)
    };

    let mut total_net: u64 = 0;
    let mut total_out: u64 = 0;
    let mut total_fee: u64 = 0;
    let mut bins_traversed: u64 = 0;

    for info in ctx.remaining_accounts.iter() {
        let mut data = info.try_borrow_mut_data()?;
        let bin_array: &mut BinArray = from_bytes_mut(&mut data[8..]);

        if !bin_array.contains(cur_bin_id) {
            continue;
        }

        let mut price = bin_to_price(cur_bin_id, bin_step_bps)?;

        loop {
            if (params.a_to_b && cur_bin_id >= params.price_limit_bin_id)
                || (!params.a_to_b && cur_bin_id <= params.price_limit_bin_id)
            {
                break;
            }

            let bin = match bin_array.get_bin_mut(cur_bin_id) {
                Ok(b) => b,
                Err(_) => break,
            };

            if params.exact_in {
                let used_gross = total_net.saturating_add(total_fee);
                if used_gross >= params.amount {
                    break;
                }
                let remaining_gross = params.amount.saturating_sub(used_gross);
                let max_net = ((remaining_gross as u128)
                    .checked_mul(10000u128)
                    .ok_or(error!(DlmmError::MathOverflow))?
                    .checked_div((10000u128).saturating_add(fee_bps as u128))
                    .ok_or(error!(DlmmError::DivisionByZero))?) as u64;

                if max_net == 0 {
                    break;
                }

                let step = compute_swap_step(
                    max_net,
                    bin.amount_x,
                    bin.amount_y,
                    price,
                    params.a_to_b,
                    fee_bps,
                )?;

                if step.amount_out == 0 && !step.bin_depleted {
                    cur_bin_id = cur_bin_id.saturating_add(direction);
                    price = q64_mul(price, step_multiplier)?;
                    bins_traversed = bins_traversed.saturating_add(1);
                    continue;
                }

                if params.a_to_b {
                    bin.amount_y = bin
                        .amount_y
                        .checked_sub(step.amount_out)
                        .ok_or(error!(DlmmError::MathOverflow))?;
                } else {
                    bin.amount_x = bin
                        .amount_x
                        .checked_sub(step.amount_out)
                        .ok_or(error!(DlmmError::MathOverflow))?;
                }

                total_net = total_net
                    .checked_add(step.amount_in_consumed)
                    .ok_or(error!(DlmmError::MathOverflow))?;
                total_out = total_out
                    .checked_add(step.amount_out)
                    .ok_or(error!(DlmmError::MathOverflow))?;
                total_fee = total_fee
                    .checked_add(step.fee_paid)
                    .ok_or(error!(DlmmError::MathOverflow))?;
                bins_traversed = bins_traversed.saturating_add(1);

                if step.bin_depleted {
                    cur_bin_id = cur_bin_id.saturating_add(direction);
                    price = q64_mul(price, step_multiplier)?;
                } else {
                    break;
                }
            } else {
                let remaining_out = params.amount.saturating_sub(total_out);
                if remaining_out == 0 {
                    break;
                }

                let available = if params.a_to_b { bin.amount_y } else { bin.amount_x };

                if available == 0 {
                    cur_bin_id = cur_bin_id.saturating_add(direction);
                    price = q64_mul(price, step_multiplier)?;
                    bins_traversed = bins_traversed.saturating_add(1);
                    continue;
                }

                if params.a_to_b {
                    let net_needed = ((remaining_out as u128)
                        .checked_mul(Q64)
                        .ok_or(error!(DlmmError::MathOverflow))?
                        .checked_div(price)
                        .ok_or(error!(DlmmError::DivisionByZero))?) as u64;

                    if remaining_out <= available {
                        let fee = apply_bps(net_needed, fee_bps)?;
                        bin.amount_y = bin.amount_y.checked_sub(remaining_out).ok_or(error!(DlmmError::MathOverflow))?;
                        total_net = total_net.checked_add(net_needed).ok_or(error!(DlmmError::MathOverflow))?;
                        total_out = total_out.checked_add(remaining_out).ok_or(error!(DlmmError::MathOverflow))?;
                        total_fee = total_fee.checked_add(fee).ok_or(error!(DlmmError::MathOverflow))?;
                        break;
                    } else {
                        let fee = apply_bps(net_needed, fee_bps)?;
                        bin.amount_y = 0;
                        total_net = total_net.checked_add(net_needed).ok_or(error!(DlmmError::MathOverflow))?;
                        total_out = total_out.checked_add(available).ok_or(error!(DlmmError::MathOverflow))?;
                        total_fee = total_fee.checked_add(fee).ok_or(error!(DlmmError::MathOverflow))?;
                        cur_bin_id = cur_bin_id.saturating_add(direction);
                        price = q64_mul(price, step_multiplier)?;
                    }
                } else {
                    let net_needed = ((remaining_out as u128)
                        .checked_mul(price)
                        .ok_or(error!(DlmmError::MathOverflow))? >> 64) as u64;

                    if remaining_out <= available {
                        let fee = apply_bps(net_needed, fee_bps)?;
                        bin.amount_x = bin.amount_x.checked_sub(remaining_out).ok_or(error!(DlmmError::MathOverflow))?;
                        total_net = total_net.checked_add(net_needed).ok_or(error!(DlmmError::MathOverflow))?;
                        total_out = total_out.checked_add(remaining_out).ok_or(error!(DlmmError::MathOverflow))?;
                        total_fee = total_fee.checked_add(fee).ok_or(error!(DlmmError::MathOverflow))?;
                        break;
                    } else {
                        let fee = apply_bps(net_needed, fee_bps)?;
                        bin.amount_x = 0;
                        total_net = total_net.checked_add(net_needed).ok_or(error!(DlmmError::MathOverflow))?;
                        total_out = total_out.checked_add(available).ok_or(error!(DlmmError::MathOverflow))?;
                        total_fee = total_fee.checked_add(fee).ok_or(error!(DlmmError::MathOverflow))?;
                        cur_bin_id = cur_bin_id.saturating_add(direction);
                        price = q64_mul(price, step_multiplier)?;
                    }
                }
                bins_traversed = bins_traversed.saturating_add(1);
            }
        }
    }

    if params.exact_in {
        require!(total_out >= params.min_amount_out, DlmmError::MinOutputNotMet);
    } else {
        let total_in = total_net.checked_add(total_fee).ok_or(error!(DlmmError::MathOverflow))?;
        require!(total_in <= params.max_amount_in, DlmmError::MaxInputExceeded);
    }

    let amount_in = total_net.checked_add(total_fee).ok_or(error!(DlmmError::MathOverflow))?;
    let amount_out = total_out;

    {
        let pool = &mut ctx.accounts.pool;
        pool.active_bin_id = cur_bin_id;

        let clock = Clock::get()?;
        let swap_price = bin_to_price(cur_bin_id, pool.bin_step_bps)?;
        let (new_acc, new_ref, new_var) = update_volatility(
            pool.vol_accumulator,
            pool.vol_reference_price,
            swap_price,
            pool.vol_last_timestamp,
            clock.unix_timestamp,
            pool.fee_decay_interval,
            200,
        );
        pool.vol_accumulator = new_acc;
        pool.vol_reference_price = new_ref;
        pool.variable_fee_bps = new_var;
        pool.vol_last_timestamp = clock.unix_timestamp;

        if params.a_to_b {
            let protocol_share = apply_bps(total_fee, pool.protocol_fee_bps)?;
            pool.pending_protocol_fees_x = pool
                .pending_protocol_fees_x
                .checked_add(protocol_share)
                .ok_or(error!(DlmmError::MathOverflow))?;
        } else {
            let protocol_share = apply_bps(total_fee, pool.protocol_fee_bps)?;
            pool.pending_protocol_fees_y = pool
                .pending_protocol_fees_y
                .checked_add(protocol_share)
                .ok_or(error!(DlmmError::MathOverflow))?;
        }
    }

    let pool_bump = ctx.accounts.pool.bump;
    let mint_a = ctx.accounts.pool.token_mint_a;
    let mint_b = ctx.accounts.pool.token_mint_b;

    if params.a_to_b {
        if amount_in > 0 {
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.user_token_a.to_account_info(),
                        to: ctx.accounts.token_vault_a.to_account_info(),
                        authority: ctx.accounts.payer.to_account_info(),
                    },
                ),
                amount_in,
            )?;
        }
        if amount_out > 0 {
            let seeds = &[b"pool", mint_a.as_ref(), mint_b.as_ref(), &[pool_bump]];
            let signer_seeds = &[&seeds[..]];
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.token_vault_b.to_account_info(),
                        to: ctx.accounts.user_token_b.to_account_info(),
                        authority: ctx.accounts.pool.to_account_info(),
                    },
                    signer_seeds,
                ),
                amount_out,
            )?;
        }
    } else {
        if amount_in > 0 {
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.user_token_b.to_account_info(),
                        to: ctx.accounts.token_vault_b.to_account_info(),
                        authority: ctx.accounts.payer.to_account_info(),
                    },
                ),
                amount_in,
            )?;
        }
        if amount_out > 0 {
            let seeds = &[b"pool", mint_b.as_ref(), mint_a.as_ref(), &[pool_bump]];
            let signer_seeds = &[&seeds[..]];
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.token_vault_a.to_account_info(),
                        to: ctx.accounts.user_token_a.to_account_info(),
                        authority: ctx.accounts.pool.to_account_info(),
                    },
                    signer_seeds,
                ),
                amount_out,
            )?;
        }
    }

    emit!(SwapEvent {
        pool: ctx.accounts.pool.key(),
        payer: ctx.accounts.payer.key(),
        a_to_b: params.a_to_b,
        amount_in,
        amount_out,
        fee: total_fee,
        active_bin_after: cur_bin_id,
        bins_traversed,
    });

    Ok(())
}
