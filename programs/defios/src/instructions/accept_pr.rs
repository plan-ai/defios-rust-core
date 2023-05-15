use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as get_associated_token_address, AssociatedToken},
    token::Token,
};

use crate::error::DefiOSError;
use crate::state::{Issue, NameRouter, PullRequest, PullRequestAccepted, Repository, VerifiedUser};

#[derive(Accounts)]
#[instruction(repo_name:String)]
pub struct AcceptPullRequest<'info> {
    #[account(
        mut,
        address = repository_verified_user.user_pubkey @ DefiOSError::UnauthorizedUser,
    )]
    pub repository_creator: Signer<'info>,
    #[account(
        seeds = [
            repository_verified_user.user_name.as_bytes(),
            repository_creator.key().as_ref(),
            name_router_account.key().as_ref()
        ],
        bump = repository_verified_user.bump
    )]
    pub repository_verified_user: Account<'info, VerifiedUser>,
    #[account(mut)]
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
        bump=pull_request_meatdata_account.bump
    )]
    pub pull_request_meatdata_account: Account<'info, PullRequest>,
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

pub fn handler(ctx: Context<AcceptPullRequest>, repo_name: String) -> Result<()> {
    let pull_request_addr = &ctx.accounts.pull_request_addr;
    let issue = &mut ctx.accounts.issue;
    let repository = &ctx.accounts.repository_account;
    let repository_creator = &ctx.accounts.repository_creator;
    let pull_request_meatdata_account = &mut ctx.accounts.pull_request_meatdata_account;
    pull_request_meatdata_account.accepted = true;
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
