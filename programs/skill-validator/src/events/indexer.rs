use anchor_lang::prelude::*;

#[event]
pub struct IndexedDataAdded {
    pub indexer: Pubkey,
    pub freelancers: Vec<Pubkey>,
    pub job: Pubkey,
}

#[event]
pub struct IndexedDataDestroyed {
    pub indexer: Pubkey,
    pub indexed_data: Pubkey,
}
