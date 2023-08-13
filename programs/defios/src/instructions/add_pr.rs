use crate::error::DefiOSError;
use crate::event::PullRequestSent;
use crate::state::{Commit, Issue, NameRouter, PullRequest, VerifiedUser};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AddPullRequest<'info> {
    #[account(mut)]
    pub pull_request_addr: Signer<'info>,
    #[account(mut)]
    pub issue: Account<'info, Issue>,
    #[account(
        init,
        payer = pull_request_addr,
        space = 8+PullRequest::INIT_SPACE,
        seeds = [
            b"pullrequestadded",
            issue.key().as_ref(),
            pull_request_addr.key().as_ref()
        ],
        bump
    )]
    pub pull_request_metadata_account: Account<'info, PullRequest>,
    #[account(
        seeds = [
            pull_request_verified_user.user_name.as_bytes(),
            pull_request_addr.key().as_ref(),
            name_router_account.key().as_ref()
        ],
        bump = pull_request_verified_user.bump
    )]
    pub pull_request_verified_user: Account<'info, VerifiedUser>,
    #[account(
        address = pull_request_verified_user.name_router @ DefiOSError::InvalidNameRouter,
        seeds = [
            name_router_account.signing_domain.as_bytes(),
            name_router_account.signature_version.to_string().as_bytes(),
            router_creator.key().as_ref()
        ],
        bump = name_router_account.bump
    )]
    pub name_router_account: Account<'info, NameRouter>,
    #[account(
        address = name_router_account.router_creator
    )]
    pub router_creator: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddPullRequest>, metadata_uri: String) -> Result<()> {
    let pull_request_addr = &ctx.accounts.pull_request_addr;
    let issue = &mut ctx.accounts.issue;
    let pull_request_metadata_account = &mut ctx.accounts.pull_request_metadata_account;

    require!(issue.closed_at.is_none(), DefiOSError::IssueClosedAlready);

    pull_request_metadata_account.bump = *ctx.bumps.get("pull_request_metadata_account").unwrap();
    pull_request_metadata_account.sent_by = pull_request_addr.key();
    pull_request_metadata_account.metadata_uri = metadata_uri.clone();
    pull_request_metadata_account.accepted = false;
    pull_request_metadata_account.commits = vec![];

    let mut commit: Account<Commit>;
    for account in ctx.remaining_accounts.to_vec().iter() {
        commit = Account::try_from(account)?;
        require!(
            pull_request_addr.key().eq(&commit.commit_creator) & issue.key().eq(&commit.issue),
            DefiOSError::UnauthorizedPR
        );
        pull_request_metadata_account.commits.push(commit.key());
    }

    if issue.first_pr_time == None {
        issue.first_pr_time = Some(Clock::get()?.unix_timestamp);
    }

    emit!(PullRequestSent {
        sent_by: pull_request_addr.key(),
        commits: pull_request_metadata_account.commits.clone(),
        metadata_uri: metadata_uri,
        issue: issue.key(),
        pull_request: pull_request_metadata_account.key()
    });

    Ok(())
}
