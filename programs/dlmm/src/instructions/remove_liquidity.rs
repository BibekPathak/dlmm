use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use bytemuck::from_bytes_mut;
use crate::state::*;
use crate::errors::DlmmError;
use crate::events::LiquidityEvent;
use crate::state::bin_array::BINS_PER_ARRAY;
use crate::instructions::add_liquidity::{ModifyLiquidity, ModifyLiquidityParams};

pub fn handler(ctx: Context<ModifyLiquidity>, params: ModifyLiquidityParams) -> Result<()> {
    require!(!params.deposits.is_empty(), DlmmError::ZeroDeposit);

    let mut seen: Vec<i32> = params.deposits.iter().map(|d| d.bin_id).collect();
    seen.sort();
    for i in 1..seen.len() {
        require!(seen[i] != seen[i - 1], DlmmError::DuplicateBinId);
    }

    let pool_key = ctx.accounts.pool.key();
    let lower = ctx.accounts.position.lower_bin_id;
    let upper = ctx.accounts.position.upper_bin_id;

    let mut total_x: u64 = 0;
    let mut total_y: u64 = 0;

    for info in ctx.remaining_accounts.iter() {
        let mut data = info.try_borrow_mut_data()?;
        let bin_array: &mut BinArray = from_bytes_mut(&mut data[8..]);

        for deposit in &params.deposits {
            if !bin_array.contains(deposit.bin_id) {
                continue;
            }
            if deposit.bin_id < lower || deposit.bin_id > upper {
                return Err(error!(DlmmError::BinIdOutOfBounds));
            }

            let bin = bin_array.get_bin_mut(deposit.bin_id)?;
            if bin.amount_x < deposit.amount_x || bin.amount_y < deposit.amount_y {
                return Err(error!(DlmmError::InsufficientLiquidity));
            }
            if deposit.amount_x == 0 && deposit.amount_y == 0 {
                return Err(error!(DlmmError::ZeroDeposit));
            }

            bin.amount_x = bin
                .amount_x
                .checked_sub(deposit.amount_x)
                .ok_or(error!(DlmmError::MathOverflow))?;
            bin.amount_y = bin
                .amount_y
                .checked_sub(deposit.amount_y)
                .ok_or(error!(DlmmError::MathOverflow))?;

            total_x = total_x
                .checked_add(deposit.amount_x)
                .ok_or(error!(DlmmError::MathOverflow))?;
            total_y = total_y
                .checked_add(deposit.amount_y)
                .ok_or(error!(DlmmError::MathOverflow))?;
        }
    }

    if total_x == 0 && total_y == 0 {
        return Err(error!(DlmmError::BinArrayNotFound));
    }

    let clock = Clock::get()?;

    {
        let position = &mut ctx.accounts.position;
        if position.total_liquidity_x < total_x || position.total_liquidity_y < total_y {
            return Err(error!(DlmmError::InsufficientLiquidity));
        }
        position.total_liquidity_x = position
            .total_liquidity_x
            .checked_sub(total_x)
            .ok_or(error!(DlmmError::MathOverflow))?;
        position.total_liquidity_y = position
            .total_liquidity_y
            .checked_sub(total_y)
            .ok_or(error!(DlmmError::MathOverflow))?;
        position.last_update = clock.unix_timestamp;
    }

    let pool_bump = ctx.accounts.pool.bump;
    let mint_a = ctx.accounts.pool.token_mint_a;
    let mint_b = ctx.accounts.pool.token_mint_b;

    if total_x > 0 {
        let seeds = &[
            b"pool",
            mint_a.as_ref(),
            mint_b.as_ref(),
            &[pool_bump],
        ];
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
            total_x,
        )?;
    }
    if total_y > 0 {
        let seeds = &[
            b"pool",
            mint_b.as_ref(),
            mint_a.as_ref(),
            &[pool_bump],
        ];
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
            total_y,
        )?;
    }

    emit!(LiquidityEvent {
        pool: pool_key,
        position: ctx.accounts.position.key(),
        owner: ctx.accounts.owner.key(),
        bin_ids: params.deposits.iter().map(|d| d.bin_id).collect(),
        amounts_x: params.deposits.iter().map(|d| d.amount_x).collect(),
        amounts_y: params.deposits.iter().map(|d| d.amount_y).collect(),
        is_deposit: false,
    });

    Ok(())
}
