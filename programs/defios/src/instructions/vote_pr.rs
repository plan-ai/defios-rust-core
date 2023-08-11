use crate::error::DefiOSError;
use crate::state::{Issue, IssueStaker, PullRequest, Repository};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

#[derive(Accounts)]
pub struct VotePRs<'info> {
    pub issue_staker: Signer<'info>,
    #[account(mut)]
    pub repository: Account<'info, Repository>,
    #[account(
        seeds = [
            b"pullrequestadded",
            issue_account.key().as_ref(),
            pull_request_metadata_account.sent_by.key().as_ref()
        ],
        bump=pull_request_metadata_account.bump
    )]
    pub pull_request_metadata_account: Account<'info, PullRequest>,
    #[account(
        seeds = [
            b"issue",
            repository.issue_index.to_string().as_bytes(),
            issue_account.repository.key().as_ref(),
            issue_account.issue_creator.key().as_ref(),
        ],
        bump
    )]
    pub issue_account: Account<'info, Issue>,
    #[account(mut)]
    pub rewards_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<VotePRs>) -> Result<()> {
    Ok(())
}
