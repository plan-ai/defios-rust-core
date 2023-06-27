use anchor_lang::prelude::*;

//to do :make of same type root and leaves
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
#[repr(u8)]
pub enum ObjectiveState {
    Locked,
    InProgress,
    Closed,
    Deprecated,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
#[repr(u8)]
pub enum ObjectiveDeliverable {
    Infrastructure,
    Tooling,
    Publication,
    Product,
    Other,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
#[repr(u8)]
pub enum RoadmapOutlook {
    Next2,
    Next5,
    Plus5,
    LongTerm,
}

#[account]
#[derive(InitSpace)]
pub struct NameRouter {
    pub bump: u8,
    pub signature_version: u8,
    pub total_verified_users: u64,
    pub router_creator: Pubkey,
    #[max_len(50)]
    pub signing_domain: String,
}

#[account]
#[derive(InitSpace)]
pub struct VerifiedUser {
    pub bump: u8,
    pub name_router: Pubkey,
    #[max_len(40)]
    pub user_name: String,
    pub user_pubkey: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct Repository {
    pub bump: u8,
    pub issue_index: u64,
    pub name_router: Pubkey,
    pub repository_creator: Pubkey,
    pub rewards_mint: Option<Pubkey>,
    #[max_len(50)]
    pub id: String,
    #[max_len(250)]
    pub description: String,
    #[max_len(100)]
    pub uri: String,
    pub vesting_schedule: Option<Pubkey>,
}

#[account]
#[derive(InitSpace)]
pub struct DefaultVestingSchedule {
    pub bump: u8,
    pub number_of_schedules: u32,
    pub per_vesting_amount: u64,
    pub unix_change: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Issue {
    pub bump: u8,
    pub index: u64,
    pub issue_creator: Pubkey,
    pub issue_token_pool_account: Pubkey,
    pub repository: Pubkey,
    pub commit_index: u64,
    pub created_at: u64,
    pub closed_at: Option<u64>,
    #[max_len(100)]
    pub uri: String,
}

#[account]
#[derive(InitSpace)]
pub struct VestingSchedule {
    pub bump: u8,
    pub destination_address: Pubkey,
    pub mint_address: Pubkey,
    #[max_len(10)]
    pub schedules: Vec<Schedule>,
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct Schedule {
    pub release_time: u64,
    pub amount: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Commit {
    pub bump: u8,
    pub index: u64,
    pub commit_creator: Pubkey,
    pub issue: Pubkey,
    #[max_len(40)]
    pub commit_hash: String,
    #[max_len(40)]
    pub tree_hash: String,
    pub created_at: u64,
    #[max_len(100)]
    pub metadata_uri: String,
}

#[account]
#[derive(InitSpace)]
pub struct IssueStaker {
    pub bump: u8,
    pub staked_amount: u64,
    #[max_len(30)]
    pub staked_at: Vec<u64>,
    pub issue_staker: Pubkey,
    pub issue: Pubkey,
    pub issue_staker_token_account: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct PullRequest {
    pub bump: u8,
    pub sent_by: Pubkey,
    #[max_len(30)]
    pub commits: Vec<Pubkey>,
    #[max_len(100)]
    pub metadata_uri: String,
    pub accepted: bool,
    pub pull_request_token_account: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct PRStaker {
    pub bump: u8,
    pub staked_amount: u64,
    #[max_len(30)]
    pub staked_at: Vec<u64>,
    pub pr_staker: Pubkey,
    pub pr: Pubkey,
    pub pr_staker_token_account: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct RoadMapMetaDataStore {
    pub bump: u8,
    #[max_len(50)]
    pub roadmap_title: String,
    pub roadmap_creation_unix: i64,
    pub roadmap_creator_id: Pubkey,
    #[max_len(100)]
    pub roadmap_description_link: String,
    #[max_len(20)]
    pub root_objective_ids: Vec<Pubkey>,
    pub roadmap_creator: Pubkey,
    pub roadmap_outlook: RoadmapOutlook,
    #[max_len(100)]
    pub roadmap_image_url: String,
    pub roadmap_repository: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct Objective {
    pub bump: u8,
    #[max_len(50)]
    pub objective_title: String,
    pub objective_creation_unix: i64,
    pub objective_creator_id: Pubkey,
    pub objective_start_unix: i64,
    pub objective_end_unix: Option<i64>,
    #[max_len(100)]
    pub objective_description_link: String,
    pub objective_state: ObjectiveState,
    #[max_len(20)]
    pub children_objective_keys: Vec<Pubkey>,
    pub objective_deliverable: ObjectiveDeliverable,
    pub objective_issue: Pubkey,
    #[max_len(50)]
    pub objective_id: String,
    pub objective_repository: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct CommunalAccount {
    pub bump: u8,
}
