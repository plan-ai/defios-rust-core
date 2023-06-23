use anchor_lang::prelude::*;

#[event]
pub struct LeafStaked {
    pub leaf: [u8; 32],
    pub index: u32,
    pub stake_amount: u64,
    pub tree: Pubkey,
}

#[event]
pub struct LeafUnStaked {
    pub leaf: [u8; 32],
    pub index: u32,
    pub unstake_amount: u64,
    pub tree: Pubkey,
}
