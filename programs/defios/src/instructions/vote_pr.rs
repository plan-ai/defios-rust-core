use crate::error::DefiOSError;
use crate::event::PRVoted;
use crate::state::{Issue, IssueStaker, PullRequest, Repository};
use anchor_lang::prelude::*;

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
        mut,
        seeds = [
            b"issue",
            issue_account.index.to_string().as_bytes(),
            repository.key().as_ref(),
            issue_account.issue_creator.key().as_ref(),
        ],
        bump=issue_account.bump
    )]
    pub issue_account: Account<'info, Issue>,
    #[account(
        mut,
        seeds = [b"issuestaker", issue_account.key().as_ref(), issue_staker.key().as_ref()],
        bump=issue_staker_account.bump
    )]
    pub issue_staker_account: Account<'info, IssueStaker>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<VotePRs>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let issue_account = &mut ctx.accounts.issue_account;
    let issue_staker_account = &mut ctx.accounts.issue_staker_account;
    let pull_request_metadata_account = &mut ctx.accounts.pull_request_metadata_account;

    require!(
        pull_request_metadata_account.accepted == false && issue_account.closed_at.is_none(),
        DefiOSError::PullRequestVotingClosedAlready
    );
    pull_request_metadata_account.total_voted_amount += issue_staker_account.pr_voting_power;
    emit!(PRVoted {
        pull_request: pull_request_metadata_account.key(),
        vote_amount: issue_staker_account.pr_voting_power,
        voter: ctx.accounts.issue_staker.key()
    });

    issue_account.total_voted_amount += issue_staker_account.pr_voting_power;
    issue_staker_account.pr_voting_power = 0;
    issue_staker_account.has_voted = true;
    issue_staker_account.voted_on = Some(pull_request_metadata_account.key());

    Ok(())
}
