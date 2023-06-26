use crate::events::indexed_data::IndexedData;
use crate::state::{freelancer::Freelancer, job::Job};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[event_cpi]
pub struct IndexData<'info> {
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
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<IndexData>, metadata_uris: Vec<String>) -> Result<()> {
    let indexer = &ctx.accounts.indexer;
    let job = &ctx.accounts.job;

    let mut added_metadata: Vec<String> = vec![];
    let mut freelancers_list: Vec<Pubkey> = vec![];
    let mut freelancer: Account<Freelancer>;
    for (index, account) in ctx.remaining_accounts.iter().enumerate() {
        freelancer = Account::try_from(account)?;
        if job.appliers.contains(&freelancer.user_pubkey) {
            added_metadata.push(metadata_uris[index].clone());
            freelancers_list.push(freelancer.user_pubkey);
        };
    }

    emit_cpi!(IndexedData {
        validator: indexer.key(),
        freelancers: freelancers_list,
        metadata_uris: added_metadata
    });

    Ok(())
}
