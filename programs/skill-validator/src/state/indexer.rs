use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct IndexedData {
    pub bump: u8,
    pub indexer: Pubkey,
    #[max_len(1000)]
    pub freelancers: Vec<Pubkey>,
    pub job: Pubkey,
}
