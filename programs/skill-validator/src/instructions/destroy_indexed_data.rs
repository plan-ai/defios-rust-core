use crate::events::indexer::IndexedDataDestroyed;
use crate::state::indexer::IndexedData;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DestroyIndexedData<'info> {
    #[account(mut)]
    pub indexer: Signer<'info>,
    #[account(
        mut,
        seeds=[
            indexer.key().as_ref(),
            indexed_data.job.key().as_ref(),
            b"indexed_job"
        ],
        close = indexer,
        bump=indexed_data.bump
    )]
    pub indexed_data: Account<'info, IndexedData>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DestroyIndexedData>) -> Result<()> {
    emit!(IndexedDataDestroyed {
        indexer: ctx.accounts.indexer.key(),
        indexed_data: ctx.accounts.indexed_data.key()
    });
    Ok(())
}
