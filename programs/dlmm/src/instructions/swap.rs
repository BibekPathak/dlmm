use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::DlmmError;

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
    pub token_vault_a: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_vault_b: UncheckedAccount<'info>,
    #[account(mut)]
    pub user_token_a: UncheckedAccount<'info>,
    #[account(mut)]
    pub user_token_b: UncheckedAccount<'info>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn handler(_ctx: Context<Swap>, _params: SwapParams) -> Result<()> {
    Err(error!(DlmmError::NotImplemented))
}
