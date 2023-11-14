use crate::error::DefiOSError;
use crate::event::AddObjectiveDataEvent;
use crate::state::{
    Objective, ObjectiveDeliverable, ObjectiveState, Repository, RoadMapMetaDataStore, VerifiedUser,
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
            objective_data_addr.key().as_ref(),
            objective_id.as_bytes()
            ],
        bump
    )]
    pub metadata_account: Account<'info, Objective>,
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
    pub objective_verified_user: Box<Account<'info, VerifiedUser>>,
    #[account(mut)]
    pub roadmap_metadata_account: Option<Account<'info, RoadMapMetaDataStore>>,
    #[account[mut]]
    pub parent_objective_account: Option<Account<'info, Objective>>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AddObjective>,
    objective_id: String,
    objective_title: String,
    objective_start_unix: i64,
    objective_description_link: String,
    objective_deliverable: ObjectiveDeliverable,
) -> Result<()> {
    let objective_creation_unix = Clock::get()?.unix_timestamp;
    let metadata_account = &mut ctx.accounts.metadata_account;
    let objective_data_addr = &mut ctx.accounts.objective_data_addr;
    let repository_account = &ctx.accounts.repository_account;
    let objective_state = ObjectiveState::InProgress;
    let roadmap_metadata_account = &mut ctx.accounts.roadmap_metadata_account;
    let parent_objective_account = &mut ctx.accounts.parent_objective_account;

    require!(
        objective_start_unix > 0,
        DefiOSError::CantEnterTimeBelowZero
    );

    metadata_account.bump = *ctx.bumps.get("metadata_account").unwrap();
    metadata_account.objective_title = objective_title.clone();
    metadata_account.objective_start_unix = objective_start_unix;
    metadata_account.objective_creation_unix = objective_creation_unix;
    metadata_account.objective_creator_id = objective_data_addr.key();
    metadata_account.next_obective_key = None;
    metadata_account.objective_description_link = objective_description_link.clone();
    metadata_account.objective_state = objective_state;
    metadata_account.objective_deliverable = objective_deliverable;
    metadata_account.objective_id = objective_id;
    metadata_account.objective_repository = repository_account.key();
    metadata_account.completed_at = None;

    let mut parent = metadata_account.key();
    match roadmap_metadata_account {
        Some(roadmap_metadata_account) => {
            require!(
                roadmap_metadata_account.root_objective == None,
                DefiOSError::InvalidObjectiveParent
            );
            parent = roadmap_metadata_account.key();
            roadmap_metadata_account.root_objective = Some(metadata_account.key())
        }
        None => match parent_objective_account {
            Some(parent_objective_account) => {
                require!(
                    parent_objective_account.next_obective_key == None,
                    DefiOSError::InvalidObjectiveParent
                );
                parent = parent_objective_account.key();
                parent_objective_account.next_obective_key = Some(metadata_account.key())
            }
            None => {
                require!(1 == 0, DefiOSError::NoParentEntered)
            }
        },
    };

    metadata_account.parent_objective = parent;
    emit!(AddObjectiveDataEvent {
        objective_title: objective_title,
        objective_metadata_uri: objective_description_link,
        objective_start_unix: objective_start_unix,
        objective_creation_unix: objective_creation_unix,
        objective_deliverable: objective_deliverable,
        objective_public_key: metadata_account.key(),
        objective_addr: objective_data_addr.key(),
        parent_objective: parent
    });

    Ok(())
}
