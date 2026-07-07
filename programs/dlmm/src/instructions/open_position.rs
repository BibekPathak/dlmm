use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::DlmmError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OpenPositionParams {
    pub lower_bin_id: i32,
    pub upper_bin_id: i32,
}

#[derive(Accounts)]
#[instruction(params: OpenPositionParams)]
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

pub fn handler(ctx: Context<OpenPosition>, params: OpenPositionParams) -> Result<()> {
    require!(params.lower_bin_id <= params.upper_bin_id, DlmmError::InvalidBinRange);

    let clock = Clock::get()?;
    let position = &mut ctx.accounts.position;
    position.owner = ctx.accounts.owner.key();
    position.pool = ctx.accounts.pool.key();
    position.lower_bin_id = params.lower_bin_id;
    position.upper_bin_id = params.upper_bin_id;
    position.total_liquidity_x = 0;
    position.total_liquidity_y = 0;
    position.fee_checkpoint_x = 0;
    position.fee_checkpoint_y = 0;
    position.fees_owed_x = 0;
    position.fees_owed_y = 0;
    position.last_update = clock.unix_timestamp;
    position.bump = ctx.bumps.position;
    Ok(())
}
