use crate::constants::AUTHORIZED_PUBLIC_KEY;
use crate::error::ApplicationError;
use crate::events::job::FreelancerAssigned;
use crate::state::{freelancer::Freelancer, job::Job};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AcceptFreelancer<'info> {
    #[account(constraint = (accepter.key().eq(&job.job_creator) || accepter.key().eq(&AUTHORIZED_PUBLIC_KEY)))]
    pub accepter: Signer<'info>,
    pub freelancer: SystemAccount<'info>,
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

pub fn handler(ctx: Context<AcceptFreelancer>) -> Result<()> {
    let job = &mut ctx.accounts.job;
    let freelancer = &ctx.accounts.freelancer;

    job.assigned_freelancer = Some(freelancer.key());

    emit!(FreelancerAssigned {
        job: job.key(),
        freelancer: freelancer.key()
    });

    Ok(())
}
