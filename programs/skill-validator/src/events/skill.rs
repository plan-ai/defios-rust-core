use anchor_lang::prelude::*;

#[event]
pub struct SkillCreated {
    pub indexes: Vec<u32>,
    pub merkle_trees: Vec<Pubkey>,
    pub freelancer: Pubkey,
    pub skill_creator: Pubkey,
}

#[event]
pub struct SkillDestroyed {
    pub skill: Pubkey,
    pub skill_creator: Pubkey,
}
