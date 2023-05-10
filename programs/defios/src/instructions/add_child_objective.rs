use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as get_associated_token_address, AssociatedToken},
    token::Token,
};

use crate::error::DefiOSError;
use crate::state::{AddChildObjectiveEvent, Objective, RoadMapMetaDataStore};

#[derive(Accounts)]
pub struct AddChildObjective<'info> {
    #[account(mut)]
    pub child_objective_adder: Signer<'info>,
    #[account(mut)]
    pub roadmap_metadata_account: Option<Account<'info, RoadMapMetaDataStore>>,
    #[account(mut)]
    pub objective_account: Account<'info, Objective>,
    #[account[mut]]
    pub parent_account: Option<Account<'info, Objective>>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddChildObjective>) -> Result<()> {
    let roadmap_metadata_account = &mut ctx.accounts.roadmap_metadata_account;
    let objective_account = &mut ctx.accounts.objective_account;
    let parent_account = &mut ctx.accounts.parent_account;
    let child_objective_adder = &mut ctx.accounts.child_objective_adder;

    match roadmap_metadata_account {
        Some(i) => {
            i.number_of_objectives = i.number_of_objectives.saturating_add(1);
            i.root_objective_ids.push(objective_account.key());
            msg!(
                "Adding objective to roadmap, objective: {}, roadmap:{}",
                objective_account.key(),
                i.key()
            );

            emit!(AddChildObjectiveEvent {
                parent_account: i.key(),
                added_by: child_objective_adder.key()
            });
        }
        None => match parent_account {
            Some(i) => {
                i.children_objective_id.push(objective_account.key());
                msg!(
                    "Adding objective to roadmap, objective: {}, roadmap:{}",
                    objective_account.key(),
                    i.key()
                );

                emit!(AddChildObjectiveEvent {
                    parent_account: i.key(),
                    added_by: child_objective_adder.key()
                });
            }
            None => {
                require!(true.eq(&false), DefiOSError::NoParentEntered);
            }
        },
    }

    Ok(())
}
