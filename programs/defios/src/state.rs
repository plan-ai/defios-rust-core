use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct NameRouter {
    pub bump: u8,
    pub signature_version: u8,
    pub total_verified_users: u64,
    pub router_creator: Pubkey,
    pub signing_domain: String,
}

impl NameRouter {
    pub fn size() -> usize {
        8 + // discriminator
            1 + // bump
            1 + // signature_version
            8 + // total_verified_users
            32 + // router_creator
            4 +
            50 // signing_domain
    }
}

#[account]
#[derive(Default)]
pub struct VerifiedUser {
    pub bump: u8,
    pub name_router: Pubkey,
    pub user_name: String,
    pub user_pubkey: Pubkey,
}

impl VerifiedUser {
    pub fn size() -> usize {
        8 + // discriminator
            1 + // bump
            32 + // name_router
            4 +
            32 + // user_name
            32 // user_pubkey
    }
}

#[account]
#[derive(Default)]
pub struct Repository {
    pub bump: u8,
    pub issue_index: u64,
    pub name_router: Pubkey,
    pub repository_creator: Pubkey,
    pub rewards_mint: Pubkey,
    pub name: String,
    pub description: String,
    pub uri: String,
    pub repository_token_pool_account: Pubkey,
}

impl Repository {
    pub fn size() -> usize {
        8 + // discriminator
            1 + // bump
            8 + // issue_index
            32 + // name_router
            32 + // repository_creator
            32 + // rewards_mint
            4 +
            32 + // name
            4 +
            100 + // description
            4 +
            200  // uri
    }
}

#[account]
#[derive(Default)]
pub struct Issue {
    pub bump: u8,
    pub index: u64,
    pub issue_creator: Pubkey,
    pub issue_token_pool_account: Pubkey,
    pub repository: Pubkey,
    pub commit_index: u64,
    pub created_at: u64,
    pub closed_at: Option<u64>,
    pub uri: String,
}

impl Issue {
    pub fn size() -> usize {
        8 + // discriminator
            1 + // bump
            8 + // index
            32 + // issue_creator
            32 + // issue_token_pool_account
            32 + // repository
            8 + // commit_index
            8 + // created_at
            1 +
            8 + // closed_at
            4 +
            200 // uri
    }
}

#[account]
#[derive(Default)]
pub struct Commit {
    pub bump: u8,
    pub index: u64,
    pub commit_creator: Pubkey,
    pub issue: Pubkey,
    pub commit_hash: String,
    pub tree_hash: String,
    pub created_at: u64,
    pub metadata_uri: String,
}

impl Commit {
    pub fn size() -> usize {
        8 + // discriminator
            1 + // bump
            8 + // index
            32 + // commit_creator
            32 + // issue
            4 +
            40 + // commit_hash
            4 +
            40 + // tree_hash
            8 + // created_at
            4 +
            200 // metadata_uri
    }
}

#[account]
#[derive(Default)]
pub struct IssueStaker {
    pub bump: u8,
    pub staked_amount: u64,
    pub staked_at: u64,
    pub issue_staker: Pubkey,
    pub issue: Pubkey,
    pub issue_staker_token_account: Pubkey,
}

impl IssueStaker {
    pub fn size() -> usize {
        8 + // discriminator
            1 + // bump
            8 + // staked_amount
            8 + // staked_at
            32 + // issue_staker
            32 + // issue
            32 // issue_staker_token_account
    }
}

#[account]
#[derive(Default)]
pub struct UserClaim {
    pub bump: u8,
    pub token_amount: u64,
    pub repository_account: Pubkey,
    pub name_router_account: Pubkey,
    pub gh_user: String,
    pub is_claimed: bool,
}

impl UserClaim {
    pub fn size() -> usize {
        8 + // discriminator
            1 + // bump
            8 + // token_amount
            32 + // repository_account
            32 + // name_router_account
            40 + // gh_user
            4 +
            1 // is_claimed
    }
}
#[event]
pub struct NameRouterCreated {
    pub router_creator: Pubkey,
    pub name_router_account: Pubkey,
}

#[event]
pub struct VerifiedUserAdded {
    pub router_creator: Pubkey,
    pub name_router_account: Pubkey,
    pub verified_user_account: Pubkey,
    pub user_name: String,
}

#[event]
pub struct CommitAdded {
    pub commit_creator: Pubkey,
    pub commit_account: Pubkey,
    pub issue_account: Pubkey,
    pub metadata_uri: String,
}

#[event]
pub struct IssueCreated {
    pub issue_creator: Pubkey,
    pub issue_account: Pubkey,
    pub repository_account: Pubkey,
    pub issue_token_pool_account: Pubkey,
    pub rewards_mint: Pubkey,
    pub uri: String,
}

#[event]
pub struct RepositoryCreated {
    pub repository_creator: Pubkey,
    pub repository_account: Pubkey,
    pub rewards_mint: Pubkey,
    pub uri: String,
    pub name: String,
    pub description: String,
    pub gh_usernames: Vec<String>,
    pub claim_amounts: Vec<u64>
}

#[event]
pub struct IssueStaked {
    pub issue_staker: Pubkey,
    pub issue_staker_token_account: Pubkey,
    pub issue_account: Pubkey,
    pub staked_amount: u64,
    pub rewards_mint: Pubkey,
    pub issue_contribution_link: String
}

#[event]
pub struct IssueUnstaked {
    pub issue_staker: Pubkey,
    pub issue_staker_token_account: Pubkey,
    pub issue_account: Pubkey,
    pub unstaked_amount: u64,
    pub rewards_mint: Pubkey,
    pub issue_contribution_link: String
}
