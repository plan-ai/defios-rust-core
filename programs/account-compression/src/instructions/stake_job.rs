use crate::error::ApplicationError;
use crate::state::jobs::{JobStaked, Jobs};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as create_associated_token_account, AssociatedToken, Create},
    mint::USDC,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(stake_amount:u64)]
pub struct StakeJob<'info> {
    #[account(mut,address=job.job_creator@ApplicationError::UnauthorizedStakeAttempt)]
    pub job_addr: Signer<'info>,
    #[account(
        mut,
        constraint = job_addr_usdc_account.mint==rewards_mint.key()@ApplicationError::NonUSDCStakingNotSupported,
        constraint = job_addr_usdc_account.owner == job_addr.key() @ApplicationError::IncorrectTokenAccount,
        constraint = job_addr_usdc_account.amount > stake_amount @ApplicationError::InsufficientBalance
    )]
    pub job_addr_usdc_account: Account<'info, TokenAccount>,
    #[account(
    seeds = [
        b"boringlif",
        job_addr.key().as_ref(),
        job.job_name.as_bytes()
    ],
    bump=job.bump)
    ]
    pub job: Account<'info, Jobs>,
    /// CHECK: Check done at function level
    pub job_usdc_account: AccountInfo<'info>,
    #[account(address=USDC@ApplicationError::NonUSDCStakingNotSupported)]
    pub rewards_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<StakeJob>, stake_amount: u64) -> Result<()> {
    let job = &mut ctx.accounts.job;
    let job_usdc_account = &mut ctx.accounts.job_usdc_account;
    let job_addr_usdc_account = &mut ctx.accounts.job_addr_usdc_account;
    let job_addr = &mut ctx.accounts.job_addr;
    let rewards_mint = &ctx.accounts.rewards_mint;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;

    require!(
        stake_amount % 100 == 0,
        ApplicationError::InvalidStakeAmount
    );

    if job_usdc_account.data_is_empty() {
        create_associated_token_account(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: job_addr.to_account_info(),
                associated_token: job_usdc_account.to_account_info(),
                authority: job.to_account_info(),
                mint: rewards_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }

    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: job_addr_usdc_account.to_account_info(),
                to: job_usdc_account.to_account_info(),
                authority: job_addr.to_account_info(),
            },
        ),
        stake_amount,
    )?;

    job.job_stake += stake_amount;

    emit!(JobStaked {
        job: job.key(),
        stake_amount: stake_amount,
        unix_time: Clock::get()?.unix_timestamp
    });

    Ok(())
}
