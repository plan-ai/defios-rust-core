use crate::error::DefiOSError;
use crate::state::{
    AddRoadmapDataEvent, NameRouter, Objective, RoadMapMetaDataStore, RoadmapOutlook, VerifiedUser,
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AddMetadata<'info> {
    #[account(mut)]
    pub roadmap_data_adder: Signer<'info>,
    #[account(
        init,
        payer = roadmap_data_adder,
        space = RoadMapMetaDataStore::size(),
        seeds = [
            b"roadmapmetadataadd",
            roadmap_verified_user.key().as_ref(),
            roadmap_data_adder.key().as_ref()
        ],
        bump
    )]
    pub metadata_account: Account<'info, RoadMapMetaDataStore>,
    #[account(
        seeds = [
            roadmap_verified_user.user_name.as_bytes(),
            roadmap_data_adder.key().as_ref(),
            name_router_account.key().as_ref()
        ],
        bump = roadmap_verified_user.bump
    )]
    pub roadmap_verified_user: Account<'info, VerifiedUser>,
    #[account(
        address = roadmap_verified_user.name_router @ DefiOSError::InvalidNameRouter,
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
    ctx: Context<AddMetadata>,
    roadmap_title: String,
    roadmap_description_link: String,
    roadmap_image_url: String,
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
    metadata_account.roadmap_creation_unix = roadmap_creation_unix;
    metadata_account.roadmap_creator = roadmap_data_adder.key();
    metadata_account.root_objective_ids = vec![];
    metadata_account.roadmap_outlook = roadmap_outlook;
    metadata_account.roadmap_image_url = roadmap_image_url.clone();

    let mut objective: Account<Objective>;
    for account in ctx.remaining_accounts.to_vec().iter() {
        objective = Account::try_from(account)?;

        match objective.objective_end_unix {
            Some(child_objective_end_unix) => {
                if child_objective_end_unix > roadmap_creation_unix
                    && objective.objective_creator_id.eq(&roadmap_data_adder.key())
                {
                    metadata_account.root_objective_ids.push(objective.key());
                }
            }
            None => {
                metadata_account.root_objective_ids.push(objective.key());
            }
        }
    }
    emit!(AddRoadmapDataEvent {
        roadmap_title: roadmap_title,
        roadmap_description_link: roadmap_description_link,
        roadmap_creation_unix: roadmap_creation_unix as u64,
        roadmap_creator: roadmap_data_adder.key(),
        root_objective_ids: metadata_account.root_objective_ids.clone(),
        roadmap_outlook: roadmap_outlook,
        roadmap_image_url: roadmap_image_url,
        roadmap: metadata_account.key()
    });

    Ok(())
}
