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
        objective_description_link: String,
        objective_deliverable: ObjectiveDeliverable,
    ) -> Result<()> {
        add_objective_data::handler(
            ctx,
            objective_id,
            objective_title,
            objective_start_unix,
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

    pub fn vote_pr(ctx: Context<VotePRs>) -> Result<()> {
        vote_pr::handler(ctx)
    }

    pub fn change_repo_token(
        ctx: Context<ChangeRepoToken>,
        token_name: Box<Option<String>>,
        token_symbol: Box<Option<String>>,
        token_metadata_uri: Box<Option<String>>,
    ) -> Result<()> {
        change_repo_token::handler(ctx, token_name, token_symbol, token_metadata_uri)
    }

    pub fn grant_money(
        ctx: Context<GrantMoney>,
        transfer_amount: u64,
        grant_metadata_uri: String,
    ) -> Result<()> {
        grant_money::handler(ctx, transfer_amount, grant_metadata_uri)
    }

    pub fn disperse_grant(ctx: Context<DisperseGrant>, disperse_amount: u64) -> Result<()> {
        disperse_grant::handler(ctx, disperse_amount)
    }

    pub fn accept_issue_vote(ctx: Context<AcceptIssueVote>) -> Result<()> {
        accept_issue_vote::handler(ctx)
    }

    pub fn create_objective_proposal(
        ctx: Context<CreateObjectiveProposal>,
        proposal_id: String,
        objective_proposal_url: String,
    ) -> Result<()> {
        create_objective_proposal::handler(ctx, proposal_id, objective_proposal_url)
    }

    pub fn vote_objective(ctx: Context<VoteObjective>, positive: bool) -> Result<()> {
        vote_objective::handler(ctx, positive)
    }

    pub fn accept_objective(ctx:Context<AcceptObjective>) -> Result<()>{
        accept_objective::handler(ctx)
    }
}
