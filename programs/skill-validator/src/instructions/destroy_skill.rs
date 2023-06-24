use crate::error::ApplicationError;
use crate::events::skill::SkillDestroyed;
use crate::state::skill::Skill;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DestroySkill<'info> {
    #[account(mut,constraint=skill_creator.key()==skill.skill_creator.key()@ApplicationError::UnauthorizedSkillAction)]
    pub skill_creator: Signer<'info>,
    #[account(
        mut,
        seeds=[
            skill.skill_creator.key().as_ref(),
            skill.freelancer.key().as_ref(),
            b"skillset_lifestyle"
        ],
        constraint = skill.in_use==false@ApplicationError::CantDestorySkillInUse,
        close=skill_creator,
        bump=skill.bump
    )]
    pub skill: Account<'info, Skill>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DestroySkill>) -> Result<()> {
    emit!(SkillDestroyed {
        skill: ctx.accounts.skill.key(),
        skill_creator: ctx.accounts.skill_creator.key()
    });
    Ok(())
}
