use anchor_lang::prelude::*;
use instructions::*;

pub mod error;
pub mod instructions;
pub mod state;

declare_id!("6wAiR6rasR2eQrjWTuVQLUQ7PKVofRDQr9qxeMcj1M9W");

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
}
