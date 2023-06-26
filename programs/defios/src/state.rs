use anchor_lang::prelude::*;

//to do :make of same type root and leaves
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ObjectiveState {
    Locked,
    InProgress,
    Closed,
    Deprecated,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ObjectiveDeliverable {
    Infrastructure,
    Tooling,
    Publication,
    Product,
    Other,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
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
    pub staked_at: Vec<u64>,
    pub issue_staker: Pubkey,
    pub issue: Pubkey,
    pub issue_staker_token_account: Pubkey,
}

impl IssueStaker {
    pub fn size() -> usize {
        8 + // discriminator
            1 + // bump
            8 + // staked_amount
            240 + // staked_at
            32 + // issue_staker
            32 + // issue
            32 // issue_staker_token_account
    }
}

#[account]
#[derive(Default)]
pub struct PRStaker {
    pub bump: u8,
    pub staked_amount: u64,
    pub staked_at: Vec<u64>,
    pub pr_staker: Pubkey,
    pub pr: Pubkey,
    pub pr_staker_token_account: Pubkey,
}

impl PRStaker {
    pub fn size() -> usize {
        8 + // discriminator
            1 + // bump
            8 + // staked_amount
            240 + // staked_at
            32 + // pr_staker
            32 + // pr
            32 // pr_staker_token_account
    }
}

#[account]
pub struct RoadMapMetaDataStore {
    pub bump: u8,
    pub roadmap_title: String,
    pub roadmap_creation_unix: i64,
    pub roadmap_creator_id: Pubkey,
    pub roadmap_description_link: String,
    pub root_objective_ids: Vec<Pubkey>,
    pub roadmap_creator: Pubkey,
    pub roadmap_outlook: RoadmapOutlook,
    pub roadmap_image_url: String,
    pub roadmap_repository: Pubkey,
}

impl RoadMapMetaDataStore {
    pub fn size() -> usize {
        8 + // discriminator
        1 + //bump
        50 + // roadmap_title
        16 + // roadmap_creation_unix
        32 + //roadmap_creator_id
        640 + // root_objective_ids
        64 + // roadmap_description_link    
        32 +//roadmap_creator
        1 + //roadmap_outlook
        64 + //roadmap_image_url
        32 //roadmap_repository
    }
}

#[account]
pub struct Objective {
    pub bump: u8,
    pub objective_title: String,
    pub objective_creation_unix: i64,
    pub objective_creator_id: Pubkey,
    pub objective_start_unix: i64,
    pub objective_end_unix: Option<i64>,
    pub objective_description_link: String,
    pub objective_state: ObjectiveState,
    pub children_objective_keys: Vec<Pubkey>,
    pub objective_deliverable: ObjectiveDeliverable,
    pub objective_issue: Pubkey,
    pub objective_id: String,
    pub objective_repository: Pubkey,
}

impl Objective {
    pub fn size() -> usize {
        8 + // discriminator
        1 + //bump
        50 + // objective_title
        16 + // objective_creation_unix
        16 + // objective_end_unix
        16 + // objective_start_unix
        640 + // children_objective_keys
        32 +  //objective_creator_id
        32 + // objective_description_link 
        1 + //objective_state
        1 + //objective deliverable
        640 + //objective_staker_ids
        160 + //objective_staker_amts
        32 + //objective_issue
        40 + //objective id
        32 //objective_repository
    }
}

#[account]
pub struct PullRequest {
    pub bump: u8,
    pub sent_by: Pubkey,
    pub commits: Vec<Pubkey>,
    pub metadata_uri: String,
    pub accepted: bool,
    pub pull_request_token_account: Pubkey,
}

impl PullRequest {
    pub fn size() -> usize {
        8 + // discriminator
        1 + //bump
        960 + //sent_by
        960 + //commits
        200 + //metadata_uri
        32 //pull request token account
    }
}

#[account]
pub struct CommunalAccount {
    pub bump: u8,
}

impl CommunalAccount {
    pub fn size() -> usize {
        8 + // discriminator
        1 //bump
    }
}

#[event]
pub struct PullRequestSent {
    pub sent_by: Pubkey,
    pub commits: Vec<Pubkey>,
    pub metadata_uri: String,
    pub issue: Pubkey,
    pub pull_request: Pubkey,
}

#[event]
pub struct AddCommitToPR {
    pub commit: Vec<Pubkey>,
    pub by: Pubkey,
}

#[event]
pub struct AddChildObjectiveEvent {
    pub parent_objective_account: Pubkey,
    pub added_by: Pubkey,
    pub objectives: Vec<Pubkey>,
}

#[event]
pub struct AddObjectiveDataEvent {
    pub objective_title: String,
    pub objective_metadata_uri: String,
    pub objective_start_unix: i64,
    pub objective_creation_unix: i64,
    pub objective_end_unix: Option<i64>,
    pub objective_deliverable: ObjectiveDeliverable,
    pub objective_public_key: Pubkey,
    pub objective_issue: Pubkey,
    pub objective_addr: Pubkey,
    pub child_objectives: Vec<Pubkey>,
}

#[event]
pub struct AddRoadmapDataEvent {
    pub roadmap_title: String,
    pub roadmap_description_link: String,
    pub roadmap_creation_unix: u64,
    pub roadmap_creator: Pubkey,
    pub root_objective_ids: Vec<Pubkey>,
    pub roadmap_outlook: RoadmapOutlook,
    pub roadmap_image_url: String,
    pub roadmap: Pubkey,
    pub roadmap_repository: Pubkey,
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
    pub user_pubkey: Pubkey,
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
    pub rewards_mint: Option<Pubkey>,
    pub uri: String,
    pub id: String,
    pub description: String,
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub token_metadata_uri: Option<String>,
    pub vesting_account: Option<Pubkey>,
}

#[event]
pub struct IssueStaked {
    pub issue_staker: Pubkey,
    pub issue_staker_token_account: Pubkey,
    pub issue_account: Pubkey,
    pub staked_amount: u64,
    pub rewards_mint: Pubkey,
    pub issue_contribution_link: String,
}

#[event]
pub struct IssueUnstaked {
    pub issue_staker: Pubkey,
    pub issue_staker_token_account: Pubkey,
    pub issue_account: Pubkey,
    pub unstaked_amount: u64,
    pub rewards_mint: Pubkey,
    pub issue_contribution_link: String,
}

#[event]
pub struct PullRequestAccepted {
    pub pull_request_addr: Pubkey,
    pub repository: Pubkey,
    pub repository_name: String,
    pub issue: Pubkey,
    pub repository_creator: Pubkey,
}

#[event]
pub struct VestingScheduleChanged {
    pub repository_account: Pubkey,
    pub repository_creator: Pubkey,
    pub old_vesting_schedule: Vec<Schedule>,
    pub new_vesting_schedule: Vec<Schedule>,
}

#[event]
pub struct DefaultVestingScheduleChanged {
    pub number_of_schedules: u32,
    pub per_vesting_amount: u64,
    pub unix_change: u64,
}

#[event]
pub struct PullRequestStaked {
    pub pr_staker: Pubkey,
    pub pr_staker_token_account: Pubkey,
    pub pr_account: Pubkey,
    pub staked_amount: u64,
    pub rewards_mint: Pubkey,
    pub pr_contribution_link: String,
}

#[event]
pub struct PullRequestUnstaked {
    pub pr_staker: Pubkey,
    pub pr_staker_token_account: Pubkey,
    pub pr_account: Pubkey,
    pub staked_amount: u64,
    pub rewards_mint: Pubkey,
    pub pr_contribution_link: String,
}
