use crate::state::{ObjectiveDeliverable, RoadmapOutlook, Schedule};
use anchor_lang::prelude::*;
use instructions::*;

pub mod constants;
pub mod error;
pub mod event;
pub mod helper;
pub mod instructions;
pub mod state;

declare_id!("7DjMy7URNwjKa7xEsCxqrxdk7BkfTyvDjeZTJJHs9dpF");

#[program]
pub mod defios {
    use super::*;

    pub fn create_name_router(
        ctx: Context<CreateNameRouter>,
        signing_domain: String,
        signature_version: u8,
    ) -> Result<()> {
        create_name_router::handler(ctx, signing_domain, signature_version)
    }

    pub fn add_verified_user(
        ctx: Context<AddVerifiedUser>,
        user_name: String,
        user_pubkey: Pubkey,
        msg: Vec<u8>,
        sig: [u8; 64],
    ) -> Result<()> {
        add_verified_user::handler(ctx, user_name, user_pubkey, msg, sig)
    }

    pub fn create_repository(
        ctx: Context<CreateRepository>,
        id: String,
        description: String,
        uri: String,
        token_name: Box<Option<String>>,
        token_image: Box<Option<String>>,
        token_metadata_uri: Box<Option<String>>,
    ) -> Result<()> {
        create_repository::handler(
            ctx,
            id,
            description,
            uri,
            token_name,
            token_image,
            token_metadata_uri,
        )
    }

    pub fn add_issue(ctx: Context<AddIssue>, uri: String) -> Result<()> {
        add_issue::handler(ctx, uri)
    }

    pub fn stake_issue(ctx: Context<StakeIssue>, transfer_amount: u64) -> Result<()> {
        stake_issue::handler(ctx, transfer_amount)
    }

    pub fn unstake_issue(ctx: Context<UnstakeIssue>) -> Result<()> {
        unstake_issue::handler(ctx)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        claim_reward::handler(ctx)
    }

    pub fn add_roadmap_data(
        ctx: Context<AddMetadata>,
        roadmap_title: String,
        roadmap_description_link: String,
        roadmap_image_url: String,
        roadmap_outlook: RoadmapOutlook,
    ) -> Result<()> {
        add_roadmap_data::handler(
            ctx,
            roadmap_title,
            roadmap_description_link,
            roadmap_image_url,
            roadmap_outlook,
        )
    }

    pub fn add_objective_data(
        ctx: Context<AddObjective>,
        objective_id: String,
        objective_title: String,
        objective_start_unix: i64,
        objective_end_unix: Option<i64>,
        objective_description_link: String,
        objective_deliverable: ObjectiveDeliverable,
    ) -> Result<()> {
        add_objective_data::handler(
            ctx,
            objective_id,
            objective_title,
            objective_start_unix,
            objective_end_unix,
            objective_description_link,
            objective_deliverable,
        )
    }

    pub fn add_pr(ctx: Context<AddPullRequest>, metadata_uri: String) -> Result<()> {
        add_pr::handler(ctx, metadata_uri)
    }

    pub fn unlock_tokens(ctx: Context<UnlockTokens>) -> Result<()> {
        unlock_tokens::handler(ctx)
    }

    pub fn accept_pr(ctx: Context<AcceptPullRequest>, repo_name: String) -> Result<()> {
        accept_pr::handler(ctx, repo_name)
    }

    pub fn change_vesting_schedule(
        ctx: Context<AdminVestingScheduleShift>,
        new_vesting_schedule: Vec<Schedule>,
    ) -> Result<()> {
        change_vesting_schedule::handler(ctx, new_vesting_schedule)
    }

    pub fn create_communal_account(ctx: Context<RegisterCommunalAccount>) -> Result<()> {
        create_communal_account::handler(ctx)
    }

    pub fn buy_tokens(
        ctx: Context<BuyToken>,
        usdc_amount: u64,
        number_of_tokens: u64,
    ) -> Result<()> {
        buy_tokens::handler(ctx, usdc_amount, number_of_tokens)
    }

    pub fn sell_tokens(
        ctx: Context<SellToken>,
        usdc_amount: u64,
        number_of_tokens: u64,
    ) -> Result<()> {
        sell_tokens::handler(ctx, usdc_amount, number_of_tokens)
    }

    pub fn set_default_schedule(
        ctx: Context<AdminDefaultVestingScheduleShift>,
        number_of_schedules: u32,
        per_vesting_amount: u64,
        unix_change: u64,
    ) -> Result<()> {
        set_default_schedule::handler(ctx, number_of_schedules, per_vesting_amount, unix_change)
    }

    pub fn vote_pr(ctx: Context<VotePRs>) -> Result<()> {
        vote_pr::handler(ctx)
    }
}
