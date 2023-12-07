use crate::events::fit::FitDataAdded;
use crate::state::{fit::ValidatedFit, freelancer::Freelancer, job::Job};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[event_cpi]
pub struct ValidateFit<'info> {
    #[account(mut)]
    pub validator: Signer<'info>,
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
    #[account(
        init_if_needed,
        payer = validator,
        seeds=[
            validator.key().as_ref(),
            job.key().as_ref(),
            b"indexed_job"
        ],
        space = 8+ValidatedFit::INIT_SPACE,
        bump
    )]
    pub validated_data: Account<'info, ValidatedFit>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ValidateFit>) -> Result<()> {
    let validated_data = &mut ctx.accounts.validated_data;
    let validator = &ctx.accounts.validator;
    let job = &ctx.accounts.job;

    validated_data.bump = ctx.bumps.validated_data;
    validated_data.validator = validator.key();
    validated_data.job = job.key();

    let mut freelancer: Account<Freelancer>;
    for account in ctx.remaining_accounts.iter() {
        freelancer = Account::try_from(account)?;
        if job.appliers.contains(&freelancer.user_pubkey) {
            validated_data.freelancers.push(freelancer.user_pubkey);
        };
    }

    emit_cpi!(FitDataAdded {
        validator: validator.key(),
        freelancers: validated_data.freelancers.clone(),
        job: job.key(),
    });
    Ok(())
}
