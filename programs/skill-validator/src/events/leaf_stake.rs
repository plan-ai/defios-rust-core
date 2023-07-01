use anchor_lang::prelude::*;

#[event]
pub struct LeafStaked {
    pub index: u32,
    pub stake_amount: u64,
    pub tree: Pubkey,
}

#[event]
pub struct LeafUnStaked {
    pub index: u32,
    pub unstake_amount: u64,
    pub tree: Pubkey,
}
