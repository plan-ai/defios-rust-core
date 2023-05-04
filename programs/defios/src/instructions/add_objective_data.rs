use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as get_associated_token_address, AssociatedToken},
    token::Token,
};

use crate::state::{AddObjectiveDataEvent, Objective, ObjectiveDeliverable, ObjectiveState};

#[derive(Accounts)]
#[instruction(name: String)]
pub struct AddObjective<'info> {
    #[account(mut)]
    pub objective_data_addr: Signer<'info>,
    #[account(
        init,
        payer = objective_data_addr,
        space = Objective::size(),
        seeds = [
            b"objectivedataadd",
            metadata_account.key().as_ref(),
            objective_data_addr.key().as_ref()
        ],
        bump
    )]
    pub metadata_account: Account<'info, Objective>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AddObjective>,
    objective_title: String,
    objective_start_unix: u64,
    objective_end_unix: u64,
    objective_description_link: String,
    objective_state: ObjectiveState,
    objective_deliverable: ObjectiveDeliverable,
) -> Result<()> {
    let objective_creation_unix = Clock::get()?.unix_timestamp;
    let metadata_account = &mut ctx.accounts.metadata_account;

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
    metadata_account.objective_staker_ids = vec![];
    metadata_account.objective_staker_amts = vec![];

    emit!(AddObjectiveDataEvent {
        objective_title: objective_title,
        objective_metadata_uri: objective_description_link,
        objective_start_unix: objective_start_unix,
        objective_creation_unix: objective_creation_unix as u64,
        objective_end_unix: objective_end_unix,
        objective_deliverable: objective_deliverable,
        objective_public_key: metadata_account.key()
    });

    Ok(())
}
