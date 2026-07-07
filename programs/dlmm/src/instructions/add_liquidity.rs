use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use bytemuck::from_bytes_mut;
use crate::state::*;
use crate::errors::DlmmError;
use crate::math::price_math::bin_to_price;
use crate::math::liquidity_math::x_to_y;
use crate::events::LiquidityEvent;
use crate::state::bin_array::BINS_PER_ARRAY;

pub const PRICE_TOLERANCE: u64 = 1;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BinDeposit {
    pub bin_id: i32,
    pub amount_x: u64,
    pub amount_y: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ModifyLiquidityParams {
    pub deposits: Vec<BinDeposit>,
}

#[derive(Accounts)]
pub struct ModifyLiquidity<'info> {
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

pub fn handler(ctx: Context<ModifyLiquidity>, params: ModifyLiquidityParams) -> Result<()> {
    require!(!params.deposits.is_empty(), DlmmError::ZeroDeposit);

    let mut seen: Vec<i32> = params.deposits.iter().map(|d| d.bin_id).collect();
    seen.sort();
    for i in 1..seen.len() {
        require!(seen[i] != seen[i - 1], DlmmError::DuplicateBinId);
    }

    let pool_key = ctx.accounts.pool.key();
    let bin_step_bps = ctx.accounts.pool.bin_step_bps;
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
            if deposit.amount_x == 0 && deposit.amount_y == 0 {
                return Err(error!(DlmmError::ZeroDeposit));
            }

            let price = bin_to_price(deposit.bin_id, bin_step_bps)?;
            if deposit.amount_x > 0 && deposit.amount_y > 0 {
                let expected_y = x_to_y(deposit.amount_x, price)?;
                let diff = if deposit.amount_y > expected_y {
                    deposit.amount_y - expected_y
                } else {
                    expected_y - deposit.amount_y
                };
                if diff > PRICE_TOLERANCE {
                    return Err(error!(DlmmError::InvalidFeeConfig));
                }
            }

            let bin = bin_array.get_bin_mut(deposit.bin_id)?;
            bin.amount_x = bin
                .amount_x
                .checked_add(deposit.amount_x)
                .ok_or(error!(DlmmError::MathOverflow))?;
            bin.amount_y = bin
                .amount_y
                .checked_add(deposit.amount_y)
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
        position.total_liquidity_x = position
            .total_liquidity_x
            .checked_add(total_x)
            .ok_or(error!(DlmmError::MathOverflow))?;
        position.total_liquidity_y = position
            .total_liquidity_y
            .checked_add(total_y)
            .ok_or(error!(DlmmError::MathOverflow))?;
        position.last_update = clock.unix_timestamp;
    }

    if total_x > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_a.to_account_info(),
                    to: ctx.accounts.token_vault_a.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            total_x,
        )?;
    }
    if total_y > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_b.to_account_info(),
                    to: ctx.accounts.token_vault_b.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
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
        is_deposit: true,
    });

    Ok(())
}
