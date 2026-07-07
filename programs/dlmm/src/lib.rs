use anchor_lang::prelude::*;

pub mod state;
pub mod math;
pub mod errors;
pub mod events;
pub mod instructions;

use errors::*;
use instructions::*;

declare_id!("So11111111111111111111111111111111111111112");

#[program]
pub mod dlmm {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        params: InitializePoolParams,
    ) -> Result<()> {
        instructions::initialize_pool::handler(ctx, params)
    }

    pub fn initialize_bin_array(
        ctx: Context<InitializeBinArray>,
        start_bin_id: i32,
    ) -> Result<()> {
        instructions::initialize_bin_array::handler(ctx, start_bin_id)
    }

    pub fn open_position(
        ctx: Context<OpenPosition>,
        params: OpenPositionParams,
    ) -> Result<()> {
        instructions::open_position::handler(ctx, params)
    }

    pub fn add_liquidity(
        ctx: Context<ModifyLiquidity>,
        params: ModifyLiquidityParams,
    ) -> Result<()> {
        instructions::add_liquidity::handler(ctx, params)
    }

    pub fn remove_liquidity(
        ctx: Context<ModifyLiquidity>,
        params: ModifyLiquidityParams,
    ) -> Result<()> {
        instructions::remove_liquidity::handler(ctx, params)
    }

    pub fn swap(
        ctx: Context<Swap>,
        params: SwapParams,
    ) -> Result<()> {
        instructions::swap::handler(ctx, params)
    }

    pub fn collect_fees(
        ctx: Context<CollectFees>,
    ) -> Result<()> {
        instructions::collect_fees::handler(ctx)
    }
}
