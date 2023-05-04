use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as get_associated_token_address, AssociatedToken},
    token::Token,
};

use crate::state::{Commit, Issue, PullRequest, PullRequestSent};

#[derive(Accounts)]
pub struct AddPullRequest<'info> {
    #[account(mut)]
    pub pull_request_addr: Signer<'info>,
    #[account(mut)]
    pub issue: Account<'info, Issue>,
    #[account(constraint = pull_request_addr.key() == commit.commit_creator)]
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

    emit!(PullRequestSent {
        sent_by: vec![pull_request_addr.key()],
        commits: vec![commit.key()],
        metadata_uri: metadata_uri
    });

    Ok(())
}
