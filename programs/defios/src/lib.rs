use anchor_lang::prelude::*;
use instructions::*;
use crate::state::{ObjectiveState, ObjectiveDeliverable, RoadmapOutlook};

pub mod error;
pub mod instructions;
pub mod state;

declare_id!("7U6tVXS1qX4WuhN2C8QgxvBboonFKjHSXDQ4jkgpgbma");

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
        name: String,
        description: String,
        uri: String,
        gh_usernames: Vec<String>,
        claim_amounts: Vec<u64>,
    ) -> Result<()> {
        create_repository::handler(ctx, name, description, uri, gh_usernames, claim_amounts)
    }

    pub fn add_user_claim(
        ctx: Context<AddUserClaim>,
        user_name: String,
        amount: u64,
    ) -> Result<()> {
        add_user_claim::handler(ctx, user_name, amount)
    }

    pub fn claim_user_tokens(ctx: Context<ClaimUserTokens>, user_name: String) -> Result<()> {
        claim_tokens::handler(ctx, user_name)
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

    pub fn add_commit(
        ctx: Context<AddCommit>,
        commit_hash: String,
        tree_hash: String,
        metadata_uri: String,
    ) -> Result<()> {
        add_commit::handler(ctx, commit_hash, tree_hash, metadata_uri)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        claim_reward::handler(ctx)
    }
    
    pub fn add_roadmap_data(
        ctx: Context<AddMetadata>,
        roadmap_title: String,
        roadmap_description_link:String,
        roadmap_outlook:RoadmapOutlook
    ) -> Result<()> {
        add_roadmap_data::handler(ctx, roadmap_title, roadmap_description_link, roadmap_outlook)
    }

    pub fn send_funds(
        ctx: Context<StakeObjective>,
        transfer_amount: u64
    ) -> Result<()> {
        send_funds::handler(ctx, transfer_amount)
    }

    pub fn add_objective_data(
        ctx: Context<AddObjective>, 
        objective_title: String,
        objective_start_unix:u64,
        objective_end_unix:u64,
        objective_description_link:String,
        objective_state:ObjectiveState,
        objective_deliverable:ObjectiveDeliverable,
    ) -> Result<()> {
        add_objective_data::handler(ctx, objective_title,objective_start_unix,objective_end_unix,objective_description_link,objective_state,objective_deliverable)
    }
    
    pub fn add_child_objective(
        ctx: Context<AddChildObjective>, 
        from_root:bool
    ) -> Result<()> {
        add_child_objective::handler(ctx,from_root)
    }

    pub fn cast_vote(
       ctx: Context<CastVote>
    ) -> Result<()>{
        cast_vote::handler(ctx)
    }

    pub fn add_pr(
        ctx: Context<AddPullRequest>,
        metadata_uri:String
     ) -> Result<()>{
         add_pr::handler(ctx,metadata_uri)
     }

     pub fn add_commit_to_pr(
        ctx: Context<AddCommitToPullRequest>,
     ) -> Result<()>{
        add_commit_to_pr::handler(ctx)
     }
}