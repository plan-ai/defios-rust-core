use crate::{
    error::DefiOSError,
    event::IssueMergedByVote,
    state::{Issue, PullRequest, Repository},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AcceptIssueVote<'info> {
    pub initiator: Signer<'info>,
    #[account(
        mut,
        address = issue.repository,
        seeds = [
            b"repository",
            repository_account.id.as_bytes(),
            repository_account.repository_creator.key().as_ref(),
        ],
        bump = repository_account.bump
    )]
    pub repository_account: Account<'info, Repository>,
    #[account(mut)]
    pub issue: Account<'info, Issue>,
    #[account(
        mut,
        seeds = [
            b"pullrequestadded",
            issue.key().as_ref(),
            pull_request_metadata_account.sent_by.as_ref()
        ],
        constraint = pull_request_metadata_account.accepted == false,
        bump = pull_request_metadata_account.bump
    )]
    pub pull_request_metadata_account: Account<'info, PullRequest>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AcceptIssueVote>) -> Result<()> {
    let pull_request_metadata_account = &mut ctx.accounts.pull_request_metadata_account;
    let issue = &mut ctx.accounts.issue;
    let repository = &mut ctx.accounts.repository_account;

    let majority_threshhold = issue.total_stake_amount / 2;
    require!(
        pull_request_metadata_account.total_voted_amount > majority_threshhold,
        DefiOSError::NotEnoughVotesForIssueMerge
    );

    let timestamp = u64::from_ne_bytes(Clock::get()?.unix_timestamp.to_ne_bytes());
    issue.closed_at = Some(timestamp);

    pull_request_metadata_account.accepted = true;

    repository.num_open_issues -= 1;

    emit!(IssueMergedByVote {
        issue: issue.key(),
        pull_request_metadata_account: pull_request_metadata_account.key()
    });

    Ok(())
}
