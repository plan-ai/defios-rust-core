use anchor_lang::prelude::*;

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
    pub repository_creator: Pubkey,
    #[max_len(50)]
    pub id: String,
    #[max_len(250)]
    pub description: String,
    #[max_len(100)]
    pub uri: String,
    pub vesting_schedule: Option<Pubkey>,
    pub repo_token: Pubkey,
    pub new_token: bool,
    pub num_changes: u8,
    pub num_open_issues: u32,
    pub objectives_open: u32,
}

#[account]
#[derive(InitSpace)]
pub struct Issue {
    pub bump: u8,
    pub index: u64,
    pub issue_creator: Pubkey,
    pub repository: Pubkey,
    pub created_at: i64,
    pub closed_at: Option<i64>,
    #[max_len(100)]
    pub uri: String,
    pub first_pr_time: Option<i64>,
    pub issue_token: Pubkey,
    pub total_stake_amount: u64,
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
pub struct IssueStaker {
    pub bump: u8,
    pub staked_amount: u64,
    pub issue_staker: Pubkey,
    pub issue: Pubkey,
    pub issue_staker_token_account: Pubkey,
    pub pr_voting_power: u64,
    pub voted_on: Option<Pubkey>,
    pub has_voted: bool,
}

#[account]
#[derive(InitSpace)]
pub struct PullRequest {
    pub bump: u8,
    pub sent_by: Pubkey,
    #[max_len(100)]
    pub metadata_uri: String,
    pub accepted: bool,
    pub total_voted_amount: u64,
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
    pub root_objective: Option<Pubkey>,
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
    #[max_len(100)]
    pub objective_description_link: String,
    pub objective_state: ObjectiveState,
    pub next_obective_key: Option<Pubkey>,
    pub parent_objective: Pubkey,
    pub objective_deliverable: ObjectiveDeliverable,
    #[max_len(50)]
    pub objective_id: String,
    pub total_grant: u64,
    pub total_dispersed_grant: u64,
    pub objective_repository: Pubkey,
    pub completed_at: Option<i64>,
}

#[account]
#[derive(InitSpace)]
pub struct CommunalAccount {
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Grantee {
    pub bump: u8,
    pub grantee: Pubkey,
    pub objective: Pubkey,
    pub staked_amount: u64,
    #[max_len(100)]
    pub grant_metadata_uri: String,
}

#[account]
#[derive(InitSpace)]
pub struct ObjectiveProposal {
    pub bump: u8,
    #[max_len(50)]
    pub proposal_id: String,
    pub proposee: Pubkey,
    pub objective: Pubkey,
    #[max_len(100)]
    pub proposal_metadata_uri: String,
    pub proposed_at: i64,
    pub vote_amount: u64,
    pub deny_amount: u64,
}

#[account]
#[derive(InitSpace)]
pub struct ObjectiveProposalVote{
    pub voted_amount: u64,
    pub state: bool   
}