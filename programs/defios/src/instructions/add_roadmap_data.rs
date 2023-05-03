use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as get_associated_token_address, AssociatedToken},
    token::Token,
};

use crate::state::{AddRoadmapDataEvent, RoadMapMetaDataStore, RoadmapOutlook};

#[derive(Accounts)]
#[instruction(name: String)]
pub struct AddMetadata<'info> {
    #[account(mut)]
    pub roadmap_data_adder: Signer<'info>,
    #[account(
        init,
        payer = roadmap_data_adder,
        space = RoadMapMetaDataStore::size(),
        seeds = [
            b"roadmapmetadataadd",
            metadata_account.key().as_ref(),
            roadmap_data_adder.key().as_ref()
        ],
        bump
    )]
    pub metadata_account: Account<'info, RoadMapMetaDataStore>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<AddMetadata>,
    roadmap_title: String,
    roadmap_description_link: String,
    roadmap_outlook: RoadmapOutlook,
) -> Result<()> {
    let roadmap_creation_unix = Clock::get()?.unix_timestamp;
    let metadata_account = &mut ctx.accounts.metadata_account;
    let roadmap_data_adder = &mut ctx.accounts.roadmap_data_adder;
    msg!(
        "Adding roadmap: Title:{}, Description: {}",
        roadmap_title,
        roadmap_description_link
    );

    metadata_account.bump = *ctx.bumps.get("metadata_account").unwrap();
    metadata_account.roadmap_title = roadmap_title.clone();
    metadata_account.roadmap_description_link = roadmap_description_link.clone();
    metadata_account.number_of_objectives = 0 as u64;
    metadata_account.roadmap_creation_unix = roadmap_creation_unix as u64;
    metadata_account.roadmap_creator = roadmap_data_adder.key();
    metadata_account.root_objective_ids = vec![];
    metadata_account.roadmap_outlook = roadmap_outlook;

    emit!(AddRoadmapDataEvent {
        roadmap_title: roadmap_title,
        roadmap_description_link: roadmap_description_link,
        roadmap_creation_unix: roadmap_creation_unix as u64,
        roadmap_creator: roadmap_data_adder.key()
    });

    Ok(())
}