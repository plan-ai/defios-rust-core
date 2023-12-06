use crate::event::AddRoadmapDataEvent;
use crate::state::{Repository, RoadMapMetaDataStore, RoadmapOutlook, VerifiedUser};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AddMetadata<'info> {
    #[account(mut, address = repository_account.repository_creator.key())]
    pub roadmap_data_adder: Signer<'info>,
    #[account(
        init,
        payer = roadmap_data_adder,
        space = 8+RoadMapMetaDataStore::INIT_SPACE,
        seeds = [
            b"roadmapmetadataadd",
            repository_account.key().as_ref(),
            roadmap_data_adder.key().as_ref()
        ],
        bump
    )]
    pub metadata_account: Account<'info, RoadMapMetaDataStore>,
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
            roadmap_verified_user.user_name.as_bytes(),
            roadmap_data_adder.key().as_ref(),
            roadmap_verified_user.name_router.key().as_ref()
        ],
        bump = roadmap_verified_user.bump
    )]
    pub roadmap_verified_user: Account<'info, VerifiedUser>,
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
    let repository_account = &ctx.accounts.repository_account;

    metadata_account.bump = ctx.bumps.metadata_account;
    metadata_account.roadmap_title = roadmap_title.clone();
    metadata_account.roadmap_description_link = roadmap_description_link.clone();
    metadata_account.roadmap_creation_unix = roadmap_creation_unix;
    metadata_account.roadmap_creator = roadmap_data_adder.key();
    metadata_account.root_objective = None;
    metadata_account.roadmap_outlook = roadmap_outlook;
    metadata_account.roadmap_image_url = roadmap_image_url.clone();
    metadata_account.roadmap_repository = repository_account.key().clone();

    emit!(AddRoadmapDataEvent {
        roadmap_title: roadmap_title,
        roadmap_description_link: roadmap_description_link,
        roadmap_creation_unix: roadmap_creation_unix as u64,
        roadmap_creator: roadmap_data_adder.key(),
        root_objective_ids: metadata_account.root_objective.clone(),
        roadmap_outlook: roadmap_outlook,
        roadmap_image_url: roadmap_image_url,
        roadmap: metadata_account.key(),
        roadmap_repository: repository_account.key()
    });

    Ok(())
}
