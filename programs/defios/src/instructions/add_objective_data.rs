use crate::error::DefiOSError;
use crate::event::AddObjectiveDataEvent;
use crate::state::{
    Issue, Objective, ObjectiveDeliverable, ObjectiveState, Repository, VerifiedUser,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(objective_id: String)]
pub struct AddObjective<'info> {
    #[account(mut, address = repository_account.repository_creator.key())]
    pub objective_data_addr: Signer<'info>,
    #[account(
        init,
        payer = objective_data_addr,
        space = 8+Objective::INIT_SPACE,
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
        mut,
        seeds = [
            b"repository",
            repository_account.id.as_bytes(),
            repository_account.repository_creator.key().as_ref(),
        ],
        bump = repository_account.bump
    )]
    pub repository_account: Account<'info, Repository>,
    #[account(
        seeds = [
            objective_verified_user.user_name.as_bytes(),
            objective_data_addr.key().as_ref(),
            objective_verified_user.name_router.key().as_ref()
        ],
        bump = objective_verified_user.bump
    )]
    pub objective_verified_user: Account<'info, VerifiedUser>,
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
    let repository_account = &ctx.accounts.repository_account;
    let objective_state = ObjectiveState::InProgress;

    require!(
        objective_start_unix > 0,
        DefiOSError::CantEnterTimeBelowZero
    );

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
    metadata_account.objective_repository = repository_account.key();

    let mut objective: Account<Objective>;
    for account in ctx.remaining_accounts.to_vec().iter() {
        objective = Account::try_from(account)?;

        if objective.objective_repository.key() != repository_account.key()
            || !objective
                .objective_creator_id
                .eq(&objective_data_addr.key())
        {
            continue;
        };

        match objective.objective_end_unix {
            Some(child_objective_end_unix) => {
                if child_objective_end_unix > objective_creation_unix {
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
        child_objectives: metadata_account.children_objective_keys.clone(),
    });

    Ok(())
}
