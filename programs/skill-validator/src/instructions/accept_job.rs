use crate::error::ApplicationError;
use crate::state::job::{Job, JobAccepted};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AcceptJob<'info> {
    #[account(mut,address = job.job_creator)]
    pub job_addr: Signer<'info>,
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

pub fn handler(ctx: Context<AcceptJob>) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(
        job.assigned_freelancer != None,
        ApplicationError::NoFreelancerSelected
    );

    job.job_completed = true;

    emit!(JobAccepted { job: job.key() });

    Ok(())
}
