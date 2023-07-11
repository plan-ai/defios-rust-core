use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LeafStake {
    pub bump: u8,
    pub root: [u8; 32],
    pub leaf: [u8; 32],
    pub index: u32,
    pub stake_amount: u64,
    pub tree: Pubkey,
}
