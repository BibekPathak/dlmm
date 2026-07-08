use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use bytemuck::{from_bytes, from_bytes_mut};
use crate::state::*;
use crate::errors::DlmmError;
use crate::state::bin_array::BINS_PER_ARRAY;

#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner @ DlmmError::InvalidPositionOwner, has_one = pool)]
    pub position: Account<'info, Position>,
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

pub fn handler(ctx: Context<CollectFees>) -> Result<()> {
    let position = &ctx.accounts.position;
    let bin_step_bps = ctx.accounts.pool.bin_step_bps;
    let pool_key = ctx.accounts.pool.key();
    let position_lower = position.lower_bin_id;
    let position_upper = position.upper_bin_id;
    let pos_liq_x = position.total_liquidity_x;
    let pos_liq_y = position.total_liquidity_y;
    let pos_checkpoint_x = position.fee_checkpoint_x;
    let pos_checkpoint_y = position.fee_checkpoint_y;
    drop(position);

    let mut pending_fee_x: u64 = 0;
    let mut pending_fee_y: u64 = 0;

    for info in ctx.remaining_accounts.iter() {
        let data = info.try_borrow_data()?;
        let bin_array: &BinArray = from_bytes(&data[8..]);

        let arr_start = bin_array.start_bin_id;
        let arr_end = arr_start + BINS_PER_ARRAY as i32 - 1;
        if arr_end < position_lower || arr_start > position_upper {
            continue;
        }

        let range_start = arr_start.max(position_lower);
        let range_end = arr_end.min(position_upper);

        for bin_id in range_start..=range_end {
            let bin = bin_array.get_bin(bin_id)?;

            if bin.fee_x > 0 {
                let share_x = if bin.amount_x > 0 && pos_liq_x > 0 {
                    let raw = (pos_liq_x as u128)
                        .checked_mul(bin.fee_x as u128)
                        .ok_or(error!(DlmmError::MathOverflow))?
                        .checked_div(bin.amount_x as u128)
                        .ok_or(error!(DlmmError::DivisionByZero))?;
                    (raw as u64).min(bin.fee_x)
                } else {
                    0
                };
                pending_fee_x = pending_fee_x
                    .checked_add(share_x)
                    .ok_or(error!(DlmmError::MathOverflow))?;
            }
            if bin.fee_y > 0 {
                let share_y = if bin.amount_y > 0 && pos_liq_y > 0 {
                    let raw = (pos_liq_y as u128)
                        .checked_mul(bin.fee_y as u128)
                        .ok_or(error!(DlmmError::MathOverflow))?
                        .checked_div(bin.amount_y as u128)
                        .ok_or(error!(DlmmError::DivisionByZero))?;
                    (raw as u64).min(bin.fee_y)
                } else {
                    0
                };
                pending_fee_y = pending_fee_y
                    .checked_add(share_y)
                    .ok_or(error!(DlmmError::MathOverflow))?;
            }
        }
    }

    {
        let position = &mut ctx.accounts.position;
        position.fees_owed_x = position
            .fees_owed_x
            .checked_add(pending_fee_x)
            .ok_or(error!(DlmmError::MathOverflow))?;
        position.fees_owed_y = position
            .fees_owed_y
            .checked_add(pending_fee_y)
            .ok_or(error!(DlmmError::MathOverflow))?;

        position.fee_checkpoint_x = position
            .fee_checkpoint_x
            .checked_add(pending_fee_x)
            .ok_or(error!(DlmmError::MathOverflow))?;
        position.fee_checkpoint_y = position
            .fee_checkpoint_y
            .checked_add(pending_fee_y)
            .ok_or(error!(DlmmError::MathOverflow))?;

        let clock = Clock::get()?;
        position.last_update = clock.unix_timestamp;
    }

    let pool_bump = ctx.accounts.pool.bump;
    let mint_a = ctx.accounts.pool.token_mint_a;
    let mint_b = ctx.accounts.pool.token_mint_b;

    if pending_fee_x > 0 {
        let seeds = &[b"pool", mint_a.as_ref(), mint_b.as_ref(), &[pool_bump]];
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
            pending_fee_x,
        )?;
    }
    if pending_fee_y > 0 {
        let seeds = &[b"pool", mint_b.as_ref(), mint_a.as_ref(), &[pool_bump]];
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
            pending_fee_y,
        )?;
    }

    Ok(())
}
