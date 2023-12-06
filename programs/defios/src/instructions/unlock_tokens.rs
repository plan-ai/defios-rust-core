use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::{
    error::DefiOSError,
    state::{Repository, VestingSchedule},
};

#[derive(Accounts)]
pub struct UnlockTokens<'info> {
    #[account(
        mut,
        address = repository_account.repository_creator.key() @ DefiOSError::UnauthorizedUser,
    )]
    pub repository_creator: Signer<'info>,

    #[account(
        mut,
        constraint = repository_creator_token_account.mint.eq(&token_mint.key()),
        address = vesting_account.destination_address,
    )]
    pub repository_creator_token_account: Account<'info, TokenAccount>,

    #[account(
        address = vesting_account.mint_address,
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(
        seeds = [
            b"repository",
            repository_account.id.as_bytes(),
            repository_creator.key().as_ref(),
        ],
        bump=repository_account.bump
    )]
    pub repository_account: Account<'info, Repository>,

    #[account(
        mut,
        seeds = [
            b"vesting",
            repository_account.key().as_ref(),
        ],
        constraint = vesting_account.mint_address == repository_account.repo_token,
        bump = vesting_account.bump
    )]
    pub vesting_account: Account<'info, VestingSchedule>,

    #[account(
        mut,
        constraint = vesting_token_account.mint.eq(&token_mint.key()),
        constraint = vesting_token_account.owner.eq(&vesting_account.key())
    )]
    pub vesting_token_account: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<UnlockTokens>) -> Result<()> {
    let vesting_account = &mut ctx.accounts.vesting_account;
    let repository_creator = &mut ctx.accounts.repository_creator;
    let repository_account = &ctx.accounts.repository_account;
    let token_program = &ctx.accounts.token_program;
    let repository_creator_token_account = &ctx.accounts.repository_creator_token_account;
    let vesting_token_account = &ctx.accounts.vesting_token_account;
    let rewards_mint = &ctx.accounts.token_mint;
    let current_timestamp = Clock::get()?.unix_timestamp;

    let expected_repository_creator_token_account =
        get_associated_token_address(&repository_creator.key(), &rewards_mint.key());

    require!(
        expected_repository_creator_token_account.eq(&repository_creator_token_account.key()),
        DefiOSError::TokenAccountMismatch
    );

    let mut total_transfer_tokens = 0;
    for s in vesting_account.schedules.iter_mut() {
        if current_timestamp as u64 >= s.release_time {
            total_transfer_tokens += s.amount;
            s.amount = 0;
        }
    }

    require!(
        total_transfer_tokens > 0,
        DefiOSError::VestingNotReachedRelease
    );

    let repository_account_key = repository_account.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"vesting",
        repository_account_key.as_ref(),
        &[vesting_account.bump],
    ]];

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: vesting_token_account.to_account_info(),
                to: repository_creator_token_account.to_account_info(),
                authority: vesting_account.to_account_info(),
            },
            signer_seeds,
        ),
        total_transfer_tokens,
    )?;

    Ok(())
}
