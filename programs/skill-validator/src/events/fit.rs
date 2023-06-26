use anchor_lang::prelude::*;

#[event]
pub struct FitDataAdded {
    pub validator: Pubkey,
    pub freelancers: Vec<Pubkey>,
    pub job: Pubkey,
}
