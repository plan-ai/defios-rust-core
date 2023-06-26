use anchor_lang::prelude::*;

#[event]
pub struct IndexedData {
    pub validator: Pubkey,
    pub freelancers: Vec<Pubkey>,
    pub metadata_uris: Vec<String>,
}
