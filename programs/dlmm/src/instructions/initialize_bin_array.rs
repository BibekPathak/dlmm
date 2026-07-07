use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(start_bin_id: i32)]
pub struct InitializeBinArray<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub pool: Account<'info, Pool>,
    #[account(
        init,
        payer = payer,
        space = BinArray::LEN,
        seeds = [b"bin_array", pool.key().as_ref(), &start_bin_id.to_le_bytes()],
        bump,
    )]
    pub bin_array: AccountLoader<'info, BinArray>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeBinArray>, start_bin_id: i32) -> Result<()> {
    let mut bin_array = ctx.accounts.bin_array.load_init()?;
    bin_array.pool = ctx.accounts.pool.key();
    bin_array.start_bin_id = start_bin_id;
    bin_array.bump = ctx.bumps.bin_array;
    Ok(())
}
