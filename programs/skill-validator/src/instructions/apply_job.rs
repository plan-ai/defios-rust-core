use crate::error::ApplicationError;
use crate::events::freelancer::JobApplied;
use crate::state::{freelancer::Freelancer, job::Job};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ApplyJob<'info> {
    pub freelancer: Signer<'info>,
    #[account(
         constraint = verified_freelancer_account.user_pubkey == freelancer.key()@ApplicationError::UnauthorizedJobAction)]
    pub verified_freelancer_account: Account<'info, Freelancer>,
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
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ApplyJob>) -> Result<()> {
    let job = &mut ctx.accounts.job;
    let freelancer = &ctx.accounts.freelancer;

    job.appliers.push(freelancer.key());

    emit!(JobApplied {
        job: job.key(),
        freelancer: freelancer.key()
    });

    Ok(())
}
