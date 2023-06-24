use crate::error::ApplicationError;
use crate::events::skill::SkillCreated;
use crate::state::{freelancer::Freelancer, skill::Skill};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[event_cpi]
pub struct CreateSkill<'info> {
    #[account(mut)]
    pub skill_creator: Signer<'info>,
    #[account(
        init_if_needed,
        payer=skill_creator,
        space=8+Skill::INIT_SPACE,
        seeds=[
            skill_creator.key().as_ref(),
            freelancer.key().as_ref(),
            b"skillset_lifestyle"
        ],
        bump
    )]
    pub skill: Account<'info, Skill>,
    ///CHECK: check done in contraint
    pub freelancer: AccountInfo<'info>,
    #[account(
         constraint = verified_freelancer_account.user_pubkey == freelancer.key()@ApplicationError::UnauthorizedJobAction)]
    pub verified_freelancer_account: Account<'info, Freelancer>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateSkill>,
    roots: Box<Vec<Vec<u8>>>,
    leafs: Box<Vec<Vec<u8>>>,
    indexes: Box<Vec<u32>>,
    merkle_trees: Box<Vec<Pubkey>>,
) -> Result<()> {
    let skill_creator = &ctx.accounts.skill_creator;
    let skill = &mut ctx.accounts.skill;
    let freelancer = &ctx.accounts.freelancer;

    skill.bump = *ctx.bumps.get("skill").unwrap();
    skill.roots.extend(*roots.clone());
    skill.indexes.extend(*indexes.clone());
    skill.leafs.extend(*leafs.clone());
    skill.merkle_trees.extend(*merkle_trees.clone());
    skill.freelancer = freelancer.key();
    skill.skill_creator = skill_creator.key();
    skill.in_use = false;

    emit_cpi!(SkillCreated {
        roots: *roots,
        leafs: *leafs,
        indexes: *indexes,
        merkle_trees: *merkle_trees,
        freelancer: freelancer.key(),
        skill_creator: skill_creator.key(),
    });

    Ok(())
}
