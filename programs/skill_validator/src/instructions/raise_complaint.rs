use crate::error::ApplicationError;
use crate::state::complaint::{Complaint, ComplaintCreated};
use crate::state::job::Job;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RaiseComplaint<'info> {
    #[account(mut,address = job.job_creator@ApplicationError::NotAuthroizedToFileComplaint)]
    pub job_addr: Signer<'info>,
    #[account(
    seeds = [
        b"boringlif",
        job.job_creator.as_ref(),
        job.job_name.as_bytes()
    ],
    bump=job.bump)
    ]
    pub job: Account<'info, Job>,
    #[account(
        init,
        payer=job_addr,
        space=8+Complaint::INIT_SPACE,
        seeds=[
            b"job_complaint",
            job_addr.key().as_ref(),
            job.key().as_ref()
        ],
        bump
    )]
    pub complaint: Account<'info, Complaint>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RaiseComplaint>, complaint_text: String) -> Result<()> {
    let complaint = &mut ctx.accounts.complaint;
    let job = &ctx.accounts.job;

    complaint.bump = ctx.bumps.complaint;
    complaint.job = job.key();
    complaint.complaint = complaint_text.clone();
    complaint.accepted = false;

    emit!(ComplaintCreated {
        job: job.key(),
        complaint: complaint_text
    });

    Ok(())
}
