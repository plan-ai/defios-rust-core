use crate::state::{ObjectiveDeliverable, RoadmapOutlook, Schedule};
use anchor_lang::prelude::*;
#[event]
pub struct PullRequestSent {
    pub sent_by: Pubkey,
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
pub struct AddObjectiveDataEvent {
    pub objective_title: String,
    pub objective_metadata_uri: String,
    pub objective_start_unix: i64,
    pub objective_creation_unix: i64,
    pub objective_end_unix: Option<i64>,
    pub objective_deliverable: ObjectiveDeliverable,
    pub objective_public_key: Pubkey,
    pub objective_addr: Pubkey,
    pub parent_objective: Pubkey,
}

#[event]
pub struct AddRoadmapDataEvent {
    pub roadmap_title: String,
    pub roadmap_description_link: String,
    pub roadmap_creation_unix: u64,
    pub roadmap_creator: Pubkey,
    pub root_objective_ids: Option<Pubkey>,
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
    pub token_imported: bool,
}

#[event]
pub struct IssueStaked {
    pub issue_staker: Pubkey,
    pub issue_staker_token_account: Pubkey,
    pub issue_account: Pubkey,
    pub staked_amount: u64,
    pub rewards_mint: Pubkey,
    pub issue_contribution_link: String,
    pub staked_at: i64,
    pub pr_voting_power: u64,
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
pub struct PRVoted {
    pub pull_request: Pubkey,
    pub vote_amount: u64,
    pub voter: Pubkey,
}

#[event]
pub struct RewardClaimed {
    pub reward_claimmee: Pubkey,
    pub reward_amount: u64,
    pub pull_request: Pubkey,
}

#[event]
pub struct RepoTokenChanged {
    pub repository: Pubkey,
    pub new_token: Pubkey,
}

#[event]
pub struct GrantProvided {
    pub grantee: Pubkey,
    pub grant_amount: u64,
    pub objective: Pubkey,
    pub grant_metadata_uri: String,
}

#[event]
pub struct GrantDispersed {
    pub objective: Pubkey,
    pub issue: Pubkey,
    pub grant_amount: u64,
}
