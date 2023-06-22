use crate::error::ApplicationError;
use crate::state::jobs::{JobClosed, Jobs};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    mint::USDC,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct CloseJob<'info> {
    #[account(mut,address=job.job_creator@ApplicationError::UnauthorizedStakeAttempt)]
    pub job_addr: Signer<'info>,
    #[account(
        mut,
        constraint = job_addr_usdc_account.mint==rewards_mint.key()@ApplicationError::NonUSDCStakingNotSupported,
        constraint = job_addr_usdc_account.owner == job_addr.key() @ApplicationError::IncorrectTokenAccount,
    )]
    pub job_addr_usdc_account: Account<'info, TokenAccount>,
    #[account(
    mut,
    seeds = [
        b"boringlif",
        job_addr.key().as_ref(),
        job.job_name.as_bytes()
    ],
    bump=job.bump,
    close = job_addr)
    ]
    pub job: Account<'info, Jobs>,
    #[account(mut,close=job)]
    pub job_usdc_account: Account<'info, TokenAccount>,
    #[account(address=USDC@ApplicationError::NonUSDCStakingNotSupported)]
    pub rewards_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CloseJob>) -> Result<()> {
    let job = &ctx.accounts.job;
    let token_program = &ctx.accounts.token_program;
    let job_usdc_account = &ctx.accounts.job_usdc_account;
    let job_addr_usdc_account = &ctx.accounts.job_addr_usdc_account;
    let job_addr = &ctx.accounts.job_addr;

    let job_key = job_addr.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"boringlif",
        job_key.as_ref(),
        job.job_name.as_bytes(),
        &[job.bump],
    ]];

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: job_usdc_account.to_account_info(),
                to: job_addr_usdc_account.to_account_info(),
                authority: job.to_account_info(),
            },
            signer_seeds,
        ),
        job.job_stake,
    )?;

    emit!(JobClosed {
        job: ctx.accounts.job.key()
    });
    Ok(())
}
