use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{error::TokenVestingError, state::VestingSchedule};

#[derive(Accounts)]
pub struct UnlockTokens<'info> {
    #[account(
        address = vesting_account.authority
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        constraint = destination_token_account.mint.eq(&token_mint.key()),
        address = vesting_account.destination_address,
    )]
    pub destination_token_account: Account<'info, TokenAccount>,

    #[account(
        address = vesting_account.mint_address,
    )]
    pub token_mint: Account<'info, Mint>,

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

    #[account(
        mut,
        constraint = vesting_token_account.mint.eq(&token_mint.key()),
        constraint = vesting_token_account.owner.eq(&vesting_account.key()),
        constraint = vesting_token_account.close_authority.is_none() @ TokenVestingError::VestingAccountInvalidClose,
        constraint = vesting_token_account.delegate.is_none() @ TokenVestingError::VestingAccountInvalidDelegate,
    )]
    pub vesting_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<UnlockTokens>) -> Result<()> {
    let vesting_account = &mut ctx.accounts.vesting_account;

    let authority = &ctx.accounts.authority;

    let token_program = &ctx.accounts.token_program;
    let destination_token_account = &ctx.accounts.destination_token_account;
    let token_mint = &ctx.accounts.token_mint;
    let vesting_token_account = &ctx.accounts.vesting_token_account;

    let current_timestamp = Clock::get()?.unix_timestamp;

    let mut total_transfer_tokens = 0;
    for s in vesting_account.schedules.iter_mut() {
        if current_timestamp as u64 >= s.release_time {
            total_transfer_tokens += s.amount;
            s.amount = 0;
        }
    }

    require!(
        total_transfer_tokens > 0,
        TokenVestingError::VestingNotReachedRelease
    );

    let token_mint_key = token_mint.key();
    let authority_key = authority.key();

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"vesting",
        token_mint_key.as_ref(),
        authority_key.as_ref(),
        &[vesting_account.bump],
    ]];

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: vesting_token_account.to_account_info(),
                to: destination_token_account.to_account_info(),
                authority: vesting_account.to_account_info(),
            },
            signer_seeds,
        ),
        total_transfer_tokens,
    )?;

    Ok(())
}
