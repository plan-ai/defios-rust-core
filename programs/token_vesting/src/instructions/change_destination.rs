use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::{error::TokenVestingError, state::VestingSchedule};

#[derive(Accounts)]
pub struct ChangeDestination<'info> {
    #[account(mut)]
    pub destination_token_owner: Signer<'info>,

    #[account(
        address = vesting_account.destination_address,
        constraint = destination_token_account.owner.eq(&destination_token_owner.key()),
    )]
    pub destination_token_account: Account<'info, TokenAccount>,

    #[account(
        constraint = destination_token_account.mint.eq(&token_mint.key())
    )]
    pub new_destination_token_account: Account<'info, TokenAccount>,

    #[account(
        address = vesting_account.mint_address,
    )]
    pub token_mint: Account<'info, Mint>,

    pub authority: SystemAccount<'info>,

    #[account(
        mut,
        constraint = !vesting_account.is_initialized @ TokenVestingError::VestingAccountAlreadyInitialized,
        seeds = [
            b"vesting",
            token_mint.key().as_ref(),
            authority.key().as_ref(),
        ],
        bump = vesting_account.bump
    )]
    pub vesting_account: Account<'info, VestingSchedule>,
}

pub fn handler(ctx: Context<ChangeDestination>) -> Result<()> {
    let vesting_account = &mut ctx.accounts.vesting_account;

    let new_destination_token_account = &ctx.accounts.new_destination_token_account;

    vesting_account.destination_address = new_destination_token_account.key();

    Ok(())
}
