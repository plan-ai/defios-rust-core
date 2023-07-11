use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Skill {
    pub bump: u8,
    #[max_len(100)]
    pub indexes: Vec<u32>,
    #[max_len(100)]
    pub merkle_trees: Vec<Pubkey>,
    pub freelancer: Pubkey,
    pub skill_creator: Pubkey,
    pub in_use: bool,
}
