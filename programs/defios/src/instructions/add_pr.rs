use crate::error::DefiOSError;
use crate::event::PullRequestSent;
use crate::state::{Issue, PullRequest, VerifiedUser};
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
            pull_request_verified_user.name_router.key().as_ref()
        ],
        bump = pull_request_verified_user.bump
    )]
    pub pull_request_verified_user: Account<'info, VerifiedUser>,
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

    if issue.first_pr_time == None {
        issue.first_pr_time = Some(Clock::get()?.unix_timestamp);
    }

    emit!(PullRequestSent {
        sent_by: pull_request_addr.key(),
        metadata_uri: metadata_uri,
        issue: issue.key(),
        pull_request: pull_request_metadata_account.key()
    });

    Ok(())
}
