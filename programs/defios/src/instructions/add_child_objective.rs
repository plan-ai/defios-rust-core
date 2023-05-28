use crate::error::DefiOSError;
use crate::state::{
    AddChildObjectiveEvent, NameRouter, Objective, RoadMapMetaDataStore, VerifiedUser,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AddChildObjective<'info> {
    #[account(mut)]
    pub child_objective_adder: Signer<'info>,
    #[account(mut)]
    pub roadmap_metadata_account: Option<Account<'info, RoadMapMetaDataStore>>,
    #[account[mut]]
    pub parent_objective_account: Option<Account<'info, Objective>>,
    #[account(
        seeds = [
            objective_verified_user.user_name.as_bytes(),
            child_objective_adder.key().as_ref(),
            name_router_account.key().as_ref()
        ],
        bump = objective_verified_user.bump
    )]
    pub objective_verified_user: Account<'info, VerifiedUser>,

    #[account(
        address = objective_verified_user.name_router @ DefiOSError::InvalidNameRouter,
        seeds = [
            name_router_account.signing_domain.as_bytes(),
            name_router_account.signature_version.to_string().as_bytes(),
            router_creator.key().as_ref()
        ],
        bump = name_router_account.bump
    )]
    pub name_router_account: Account<'info, NameRouter>,
    #[account(
        address = name_router_account.router_creator
    )]
    pub router_creator: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddChildObjective>) -> Result<()> {
    let roadmap_metadata_account = &mut ctx.accounts.roadmap_metadata_account;
    let parent_objective_account = &mut ctx.accounts.parent_objective_account;
    let child_objective_adder = &mut ctx.accounts.child_objective_adder;
    let current_unix = Clock::get()?.unix_timestamp;

    let mut objective: Account<Objective>;
    match roadmap_metadata_account {
        Some(roadmap_metadata_account) => {
            for account in ctx.remaining_accounts.to_vec().iter() {
                objective = Account::try_from(account)?;

                match objective.objective_end_unix {
                    Some(child_objective_end_unix) => {
                        if child_objective_end_unix > roadmap_metadata_account.roadmap_creation_unix
                            && objective
                                .objective_creator_id
                                .eq(&child_objective_adder.key())
                        {
                            roadmap_metadata_account
                                .root_objective_ids
                                .push(objective.key());
                        }
                    }
                    None => {
                        roadmap_metadata_account
                            .root_objective_ids
                            .push(objective.key());
                    }
                }
            }

            emit!(AddChildObjectiveEvent {
                parent_objective_account: roadmap_metadata_account.key(),
                added_by: child_objective_adder.key(),
                objectives: roadmap_metadata_account.root_objective_ids.clone()
            });
        }
        None => match parent_objective_account {
            Some(parent_objective_account) => {
                for account in ctx.remaining_accounts.to_vec().iter() {
                    objective = Account::try_from(account)?;

                    match objective.objective_end_unix {
                        Some(child_objective_end_unix) => {
                            if child_objective_end_unix > current_unix
                                && objective
                                    .objective_creator_id
                                    .eq(&child_objective_adder.key())
                            {
                                parent_objective_account
                                    .children_objective_keys
                                    .push(objective.key());
                            }
                        }
                        None => {
                            parent_objective_account
                                .children_objective_keys
                                .push(objective.key());
                        }
                    }
                }

                emit!(AddChildObjectiveEvent {
                    parent_objective_account: parent_objective_account.key(),
                    added_by: child_objective_adder.key(),
                    objectives: parent_objective_account.children_objective_keys.clone()
                });
            }
            None => {
                require!(true.eq(&false), DefiOSError::NoParentEntered);
            }
        },
    }

    Ok(())
}
