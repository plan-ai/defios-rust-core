use crate::error::DefiOSError;
use crate::state::{
    AddObjectiveDataEvent, Issue, NameRouter, Objective, ObjectiveDeliverable, ObjectiveState,
    VerifiedUser,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(objective_id: String)]
pub struct AddObjective<'info> {
    #[account(mut)]
    pub objective_data_addr: Signer<'info>,
    #[account(
        init,
        payer = objective_data_addr,
        space = Objective::size(),
        seeds = [
            b"objectivedataadd",
            objective_issue.key().as_ref(),
            objective_data_addr.key().as_ref(),
            objective_id.as_bytes()
            ],
        bump
    )]
    pub metadata_account: Account<'info, Objective>,
    #[account(mut)]
    pub objective_issue: Account<'info, Issue>,
    #[account(
        seeds = [
            objective_verified_user.user_name.as_bytes(),
            objective_data_addr.key().as_ref(),
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

pub fn handler(
    ctx: Context<AddObjective>,
    objective_id: String,
    objective_title: String,
    objective_start_unix: i64,
    objective_end_unix: Option<i64>,
    objective_description_link: String,
    objective_deliverable: ObjectiveDeliverable,
) -> Result<()> {
    let objective_creation_unix = Clock::get()?.unix_timestamp;
    let metadata_account = &mut ctx.accounts.metadata_account;
    let objective_data_addr = &mut ctx.accounts.objective_data_addr;
    let objective_issue = &ctx.accounts.objective_issue;
    let objective_state = ObjectiveState::InProgress;

    match objective_end_unix {
        Some(x) => {
            require!(
                objective_creation_unix < x,
                DefiOSError::RoadmapInvalidEndTime
            );
        }
        None => {}
    }

    metadata_account.bump = *ctx.bumps.get("metadata_account").unwrap();
    metadata_account.objective_title = objective_title.clone();
    metadata_account.objective_start_unix = objective_start_unix;
    metadata_account.objective_end_unix = objective_end_unix;
    metadata_account.objective_creation_unix = objective_creation_unix;
    metadata_account.objective_creator_id = objective_data_addr.key();
    metadata_account.children_objective_keys = vec![];
    metadata_account.objective_description_link = objective_description_link.clone();
    metadata_account.objective_state = objective_state;
    metadata_account.objective_deliverable = objective_deliverable;
    metadata_account.objective_issue = objective_issue.key();
    metadata_account.objective_id = objective_id;

    let mut objective: Account<Objective>;
    for account in ctx.remaining_accounts.to_vec().iter() {
        objective = Account::try_from(account)?;

        match objective.objective_end_unix {
            Some(child_objective_end_unix) => {
                if child_objective_end_unix > objective_creation_unix
                    && objective
                        .objective_creator_id
                        .eq(&objective_data_addr.key())
                {
                    metadata_account
                        .children_objective_keys
                        .push(objective.key());
                }
            }
            None => {
                metadata_account
                    .children_objective_keys
                    .push(objective.key());
            }
        }
    }

    emit!(AddObjectiveDataEvent {
        objective_title: objective_title,
        objective_metadata_uri: objective_description_link,
        objective_start_unix: objective_start_unix,
        objective_creation_unix: objective_creation_unix,
        objective_end_unix: objective_end_unix,
        objective_deliverable: objective_deliverable,
        objective_public_key: metadata_account.key(),
        objective_issue: objective_issue.key(),
        objective_addr: objective_data_addr.key(),
        child_objectives: metadata_account.children_objective_keys.clone()
    });

    Ok(())
}
