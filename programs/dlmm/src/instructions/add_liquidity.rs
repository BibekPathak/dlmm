use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::DlmmError;

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
    pub token_vault_a: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_vault_b: UncheckedAccount<'info>,
    #[account(mut)]
    pub user_token_a: UncheckedAccount<'info>,
    #[account(mut)]
    pub user_token_b: UncheckedAccount<'info>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler(_ctx: Context<ModifyLiquidity>, _params: ModifyLiquidityParams) -> Result<()> {
    Err(error!(DlmmError::NotImplemented))
}
