use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as get_associated_token_address, AssociatedToken},
    token::Token,
};

use crate::state::{AddChildObjectiveEvent, Objective, RoadMapMetaDataStore};

#[derive(Accounts)]
#[instruction(name: String)]
pub struct AddChildObjective<'info> {
    #[account(mut)]
    pub child_objective_adder: Signer<'info>,
    #[account(mut)]
    pub roadmap_metadata_account: Account<'info, RoadMapMetaDataStore>,
    #[account(mut)]
    pub objective_account: Account<'info, Objective>,
    #[account[mut]]
    pub parent_account: Account<'info, Objective>,
    pub system_program: Program<'info, System>
}

pub fn handler(ctx: Context<AddChildObjective>, from_root: bool) -> Result<()> {
    let roadmap_metadata_account = &mut ctx.accounts.roadmap_metadata_account;
    let objective_account = &mut ctx.accounts.objective_account;
    let parent_account = &mut ctx.accounts.parent_account;
    let child_objective_adder = &mut ctx.accounts.child_objective_adder;

    msg!(
        "Adding objective to roadmap, objective: {}, roadmap:{}",
        objective_account.key(),
        roadmap_metadata_account.key()
    );

    emit!(AddChildObjectiveEvent {
            parent_account: roadmap_metadata_account.key(),
            added_by: child_objective_adder.key()
        });
    
    if from_root {
        roadmap_metadata_account.number_of_objectives = roadmap_metadata_account
            .number_of_objectives
            .saturating_add(1);
        roadmap_metadata_account
            .root_objective_ids
            .push(objective_account.key());
    
    } else {
        parent_account
            .children_objective_id
            .push(objective_account.key());
    }

    Ok(())
}