use crate::constants::{FREELANCER_SHARE, TOTAL_SHARE};
use crate::error::ApplicationError;
use crate::state::{Freelancer, Job};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as create_associated_token_account, AssociatedToken, Create},
    mint::USDC,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct ClaimFreelanceReward<'info> {
    #[account(mut)]
    pub freelancer: Signer<'info>,
    #[account(
         constraint = verified_freelancer_account.user_pubkey == freelancer.key()@ApplicationError::UnauthorizedJobAction)]
    pub verified_freelancer_account: Account<'info, Freelancer>,
    #[account(mut)]
    ///CHECK: check for this is done in function call
    pub freelancer_usdc_acc: AccountInfo<'info>,
    #[account(
        mut,
        constraint = job_usdc_acc.mint==rewards_mint.key()@ApplicationError::NonUSDCStakingNotSupported,
        constraint = job_usdc_acc.owner == job.key() @ApplicationError::IncorrectTokenAccount,
        constraint = job_usdc_acc.amount >= job.job_stake @ApplicationError::InsufficientBalance
    )]
    pub job_usdc_acc: Account<'info, TokenAccount>,
    #[account(
    mut,
    seeds = [
        b"boringlif",
        job.job_creator.as_ref(),
        job.job_name.as_bytes()
    ],
    bump=job.bump)
    ]
    pub job: Account<'info, Job>,
    #[account(address=USDC@ApplicationError::NonUSDCStakingNotSupported)]
    pub rewards_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimFreelanceReward>) -> Result<()> {
    let job = &mut ctx.accounts.job;
    let job_usdc_acc = &mut ctx.accounts.job_usdc_acc;
    let freelancer_usdc_acc = &mut ctx.accounts.freelancer_usdc_acc;
    let freelancer = &mut ctx.accounts.freelancer;
    let rewards_mint = &ctx.accounts.rewards_mint;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;
    let associated_token_program = &ctx.accounts.associated_token_program;

    let freelancer_share = (FREELANCER_SHARE * job.job_stake) / TOTAL_SHARE;
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"boringlif",
        job.job_creator.as_ref(),
        job.job_name.as_bytes(),
        &[job.bump],
    ]];
    if freelancer_usdc_acc.data_is_empty() {
        create_associated_token_account(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: freelancer.to_account_info(),
                associated_token: freelancer_usdc_acc.to_account_info(),
                authority: freelancer.to_account_info(),
                mint: rewards_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: job_usdc_acc.to_account_info(),
                to: freelancer_usdc_acc.to_account_info(),
                authority: job.to_account_info(),
            },
            signer_seeds,
        ),
        freelancer_share,
    )?;
    Ok(())
}
