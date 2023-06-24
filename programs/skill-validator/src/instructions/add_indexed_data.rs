use crate::events::indexer::IndexedDataAdded;
use crate::state::{freelancer::Freelancer, indexer::IndexedData, job::Job};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[event_cpi]
pub struct AddIndexedData<'info> {
    #[account(mut)]
    pub indexer: Signer<'info>,
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
        init,
        payer = indexer,
        seeds=[
            indexer.key().as_ref(),
            job.key().as_ref(),
            b"indexed_job"
        ],
        space = 8+IndexedData::INIT_SPACE,
        bump
    )]
    pub indexed_data: Account<'info, IndexedData>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddIndexedData>) -> Result<()> {
    let indexed_data = &mut ctx.accounts.indexed_data;
    let indexer = &ctx.accounts.indexer;
    let job = &ctx.accounts.job;

    indexed_data.bump = *ctx.bumps.get("indexed_data").unwrap();
    indexed_data.indexer = indexer.key();
    indexed_data.job = job.key();

    let mut freelancer: Account<Freelancer>;
    let mut freelancers: Vec<Pubkey> = vec![];
    for account in ctx.remaining_accounts.iter() {
        freelancer = Account::try_from(account)?;
        freelancers.push(freelancer.key());
    }

    indexed_data.freelancers = freelancers.clone();

    emit_cpi!(IndexedDataAdded {
        indexer: indexer.key(),
        freelancers: freelancers,
        job: job.key(),
    });
    Ok(())
}
