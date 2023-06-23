use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Skill {
    pub bump: u8,
    #[max_len(40, 32)]
    pub roots: Vec<Vec<u8>>,
    #[max_len(40, 32)]
    pub leafs: Vec<Vec<u8>>,
    #[max_len(40)]
    pub index: Vec<u32>,
    #[max_len(40)]
    pub merkle_trees: Vec<Pubkey>,
    pub freelancer: Pubkey,
    pub skill_creator: Pubkey,
    pub in_use: bool,
}
