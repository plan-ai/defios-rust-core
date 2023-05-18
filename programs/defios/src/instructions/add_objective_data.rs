use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as get_associated_token_address, AssociatedToken},
    token::Token,
};

use crate::error::DefiOSError;
use crate::state::{
    AddObjectiveDataEvent, Issue, NameRouter, Objective, ObjectiveDeliverable, ObjectiveState,
    VerifiedUser,
};
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
    objective_start_unix: u64,
    objective_end_unix: u64,
    objective_description_link: String,
    objective_deliverable_no: u32,
) -> Result<()> {
    let objective_creation_unix = u64::from_ne_bytes(Clock::get()?.unix_timestamp.to_ne_bytes());
    let metadata_account = &mut ctx.accounts.metadata_account;
    let objective_issue = &ctx.accounts.objective_issue;
    let objective_state = ObjectiveState::InProgress;
    let objective_deliverable: ObjectiveDeliverable;
    match objective_deliverable_no {
        1 => {
            objective_deliverable = ObjectiveDeliverable::Tooling;
        }
        2 => {
            objective_deliverable = ObjectiveDeliverable::Publication;
        }
        3 => {
            objective_deliverable = ObjectiveDeliverable::Product;
        }
        4 => {
            objective_deliverable = ObjectiveDeliverable::Other;
        }
        _ => {
            objective_deliverable = ObjectiveDeliverable::Infrastructure;
        }
    };
    require!(
        objective_creation_unix < objective_end_unix,
        DefiOSError::RoadmapInvalidEndTime
    );
    msg!(
        "Adding objective: Title:{}, Description: {}",
        objective_title,
        objective_description_link
    );

    metadata_account.bump = *ctx.bumps.get("metadata_account").unwrap();
    metadata_account.objective_title = objective_title.clone();
    metadata_account.objective_start_unix = objective_start_unix;
    metadata_account.objective_end_unix = objective_end_unix;
    metadata_account.objective_creation_unix = objective_creation_unix as u64;
    metadata_account.objective_creator_gh_id = ctx.accounts.objective_data_addr.key();
    metadata_account.children_objective_id = vec![];
    metadata_account.objective_description_link = objective_description_link.clone();
    metadata_account.objective_state = objective_state;
    metadata_account.objective_deliverable = objective_deliverable;
    metadata_account.objective_issue = objective_issue.key();
    emit!(AddObjectiveDataEvent {
        objective_title: objective_title,
        objective_metadata_uri: objective_description_link,
        objective_start_unix: objective_start_unix,
        objective_creation_unix: objective_creation_unix as u64,
        objective_end_unix: objective_end_unix,
        objective_deliverable: objective_deliverable,
        objective_public_key: metadata_account.key(),
        objective_issue: objective_issue.key()
    });

    Ok(())
}
