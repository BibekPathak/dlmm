use anchor_lang::prelude::*;

pub mod state;
pub mod errors;
pub mod math;

use errors::DlmmError;
use state::*;

declare_id!("So11111111111111111111111111111111111111112");

#[program]
pub mod dlmm {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        params: InitializePoolParams,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.authority.key();
        pool.token_mint_a = ctx.accounts.token_mint_a.key();
        pool.token_mint_b = ctx.accounts.token_mint_b.key();
        pool.token_vault_a = ctx.accounts.token_vault_a.key();
        pool.token_vault_b = ctx.accounts.token_vault_b.key();
        pool.fee_tier_bps = params.fee_tier_bps;
        pool.protocol_fee_bps = params.protocol_fee_bps;
        pool.bin_step_bps = params.bin_step_bps;
        pool.base_bin_id = params.base_bin_id;
        pool.bump = *ctx.bumps.get("pool").unwrap_or(&0);
        Ok(())
    }

    pub fn open_position(
        ctx: Context<OpenPosition>,
        params: OpenPositionParams,
    ) -> Result<()> {
        require!(params.lower_bin_id <= params.upper_bin_id, DlmmError::InvalidBinRange);
        let position = &mut ctx.accounts.position;
        position.owner = ctx.accounts.owner.key();
        position.pool = ctx.accounts.pool.key();
        position.lower_bin_id = params.lower_bin_id;
        position.upper_bin_id = params.upper_bin_id;
        position.liquidity = 0;
        position.fees_owed_a = 0;
        position.fees_owed_b = 0;
        position.bump = *ctx.bumps.get("position").unwrap_or(&0);
        Ok(())
    }

    pub fn add_liquidity(
        _ctx: Context<ModifyLiquidity>,
        _params: ModifyLiquidityParams,
    ) -> Result<()> {
        // TODO: Implement bin distribution and token transfers
        Ok(())
    }

    pub fn remove_liquidity(
        _ctx: Context<ModifyLiquidity>,
        _params: ModifyLiquidityParams,
    ) -> Result<()> {
        // TODO: Implement withdrawal and fee realization
        Ok(())
    }

    pub fn swap(_ctx: Context<Swap>, _params: SwapParams) -> Result<()> {
        // TODO: Implement price traversal across bin arrays
        Ok(())
    }

    pub fn collect_fees(_ctx: Context<CollectFees>) -> Result<()> {
        // TODO: Transfer accrued fees to position owner
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitializePoolParams {
    pub fee_tier_bps: u16,
    pub protocol_fee_bps: u16,
    pub bin_step_bps: u16,
    pub base_bin_id: i32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OpenPositionParams {
    pub lower_bin_id: i32,
    pub upper_bin_id: i32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ModifyLiquidityParams {
    pub bin_amounts: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SwapParams {
    pub amount_in: u64,
    pub a_to_b: bool,
    pub price_limit_bin_id: i32,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: pool authority chosen by user
    pub authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        space = Pool::SPACE,
        seeds = [b"pool", token_mint_a.key().as_ref(), token_mint_b.key().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,
    /// CHECK: token vaults created off-chain for now (stub)
    pub token_vault_a: UncheckedAccount<'info>,
    /// CHECK: token vaults created off-chain for now (stub)
    pub token_vault_b: UncheckedAccount<'info>,
    pub token_mint_a: AccountInfo<'info>,
    pub token_mint_b: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub pool: Account<'info, Pool>,
    #[account(
        init,
        payer = owner,
        space = Position::SPACE,
        seeds = [b"position", pool.key().as_ref(), owner.key().as_ref()],
        bump,
    )]
    pub position: Account<'info, Position>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ModifyLiquidity<'info> {
    pub owner: Signer<'info>,
    #[account(mut, has_one = pool)]
    pub position: Account<'info, Position>,
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
pub struct CollectFees<'info> {
    pub owner: Signer<'info>,
    #[account(mut, has_one = pool)]
    pub position: Account<'info, Position>,
    pub pool: Account<'info, Pool>,
}


