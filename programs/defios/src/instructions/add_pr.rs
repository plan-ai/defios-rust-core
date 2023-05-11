use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as get_associated_token_address, AssociatedToken},
    token::Token,
};

use crate::error::DefiOSError;
use crate::state::{Commit, Issue, NameRouter, PullRequest, PullRequestSent, VerifiedUser};
#[derive(Accounts)]
pub struct AddPullRequest<'info> {
    #[account(mut)]
    pub pull_request_addr: Signer<'info>,
    #[account(mut)]
    pub issue: Account<'info, Issue>,
    #[account(constraint = pull_request_addr.key().eq(&commit.commit_creator) @ DefiOSError::UnauthorizedPR)]
    pub commit: Account<'info, Commit>,
    #[account(
        init,
        payer = pull_request_addr,
        space = PullRequest::size(),
        seeds = [
            b"pullrequestadded",
            issue.key().as_ref(),
            pull_request_addr.key().as_ref()
        ],
        bump
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

pub fn handler(ctx: Context<AddPullRequest>, metadata_uri: String) -> Result<()> {
    let pull_request_addr = &ctx.accounts.pull_request_addr;
    let issue = &ctx.accounts.issue;
    let commit = &ctx.accounts.commit;
    let pull_request_meatdata_account = &mut ctx.accounts.pull_request_meatdata_account;

    msg!(
        "Adding pull request on issue {} by {}",
        issue.uri,
        pull_request_addr.key()
    );

    pull_request_meatdata_account.sent_by = vec![pull_request_addr.key()];
    pull_request_meatdata_account.commits = vec![commit.key()];
    pull_request_meatdata_account.metadata_uri = metadata_uri.clone();
    pull_request_meatdata_account.accepted = false;
    emit!(PullRequestSent {
        sent_by: vec![pull_request_addr.key()],
        commits: vec![commit.key()],
        metadata_uri: metadata_uri
    });

    Ok(())
}
