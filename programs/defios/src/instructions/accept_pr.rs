use crate::error::DefiOSError;
use crate::event::PullRequestAccepted;
use crate::state::{Issue, PullRequest, Repository};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(repo_name:String)]
pub struct AcceptPullRequest<'info> {
    #[account(
        mut,
        address = repository_account.repository_creator.key() @ DefiOSError::UnauthorizedUser,
    )]
    pub repository_creator: Signer<'info>,
    #[account(mut, address = pull_request_metadata_account.sent_by)]
    pub pull_request_addr: SystemAccount<'info>,
    #[account(
        seeds = [
            b"repository",
            repo_name.as_bytes(),
            repository_creator.key().as_ref(),
        ],
        bump=repository_account.bump
    )]
    pub repository_account: Account<'info, Repository>,
    #[account(mut,constraint = issue.repository.eq(&repository_account.key()))]
    pub issue: Account<'info, Issue>,
    #[account(
        mut,
        seeds = [
            b"pullrequestadded",
            issue.key().as_ref(),
            pull_request_addr.key().as_ref()
        ],
        bump=pull_request_metadata_account.bump
    )]
    pub pull_request_metadata_account: Account<'info, PullRequest>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AcceptPullRequest>, repo_name: String) -> Result<()> {
    let pull_request_addr = &ctx.accounts.pull_request_addr;
    let issue = &mut ctx.accounts.issue;
    let repository = &ctx.accounts.repository_account;
    let repository_creator = &ctx.accounts.repository_creator;
    let pull_request_metadata_account = &mut ctx.accounts.pull_request_metadata_account;
    pull_request_metadata_account.accepted = true;
    let timestamp = u64::from_ne_bytes(Clock::get()?.unix_timestamp.to_ne_bytes());
    issue.closed_at = Some(timestamp);

    emit!(PullRequestAccepted {
        pull_request_addr: pull_request_addr.key(),
        repository: repository.key(),
        repository_name: repo_name,
        issue: issue.key(),
        repository_creator: repository_creator.key()
    });

    Ok(())
}
