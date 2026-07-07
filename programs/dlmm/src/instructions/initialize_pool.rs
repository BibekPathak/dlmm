use anchor_lang::prelude::*;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitializePoolParams {
    pub fee_tier_bps: u16,
    pub protocol_fee_bps: u16,
    pub bin_step_bps: u16,
    pub base_bin_id: i32,
    pub active_bin_id: i32,
    pub base_fee_bps: u16,
    pub fee_decay_interval: u64,
}

#[derive(Accounts)]
#[instruction(params: InitializePoolParams)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = payer,
        space = Pool::SPACE,
        seeds = [b"pool", token_mint_a.key().as_ref(), token_mint_b.key().as_ref()],
        bump,
    )]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub token_vault_a: UncheckedAccount<'info>,
    #[account(mut)]
    pub token_vault_b: UncheckedAccount<'info>,
    pub token_mint_a: UncheckedAccount<'info>,
    pub token_mint_b: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializePool>, params: InitializePoolParams) -> Result<()> {
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
    pool.active_bin_id = params.active_bin_id;
    pool.base_fee_bps = params.base_fee_bps;
    pool.fee_decay_interval = params.fee_decay_interval;
    pool.bump = ctx.bumps.pool;
    Ok(())
}
