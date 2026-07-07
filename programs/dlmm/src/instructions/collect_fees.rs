use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::DlmmError;

#[derive(Accounts)]
pub struct CollectFees<'info> {
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

pub fn handler(_ctx: Context<CollectFees>) -> Result<()> {
    Err(error!(DlmmError::NotImplemented))
}
