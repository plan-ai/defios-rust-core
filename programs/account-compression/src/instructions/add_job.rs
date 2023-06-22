use crate::state::job::{Job, JobCreated, JobLength};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(job_name:String)]
pub struct AddJob<'info> {
    #[account(mut)]
    pub job_addr: Signer<'info>,
    #[account(init,
    space = 8+Job::INIT_SPACE,
    payer=job_addr,
    seeds = [
        b"boringlif",
        job_addr.key().as_ref(),
        job_name.as_bytes()
    ],
    bump)
    ]
    pub job: Account<'info, Job>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AddJob>,
    job_name: String,
    job_desc: String,
    job_length: JobLength,
    job_metadata_uri: String,
) -> Result<()> {
    let job = &mut ctx.accounts.job;
    let job_addr = &ctx.accounts.job_addr;

    job.bump = *ctx.bumps.get("job").unwrap();
    job.job_length = job_length;
    job.job_name = job_name.clone();
    job.job_desc = job_desc.clone();
    job.job_metadata_uri = job_metadata_uri.clone();
    job.job_creator = job_addr.key();
    job.job_stake = 0;
    job.assigned_freelancer = None;
    job.job_completed = false;

    emit!(JobCreated {
        job_length: job_length,
        job_name: job_name,
        job_desc: job_desc,
        job_metadata_uri: job_metadata_uri,
        job_creator: job_addr.key()
    });

    Ok(())
}
