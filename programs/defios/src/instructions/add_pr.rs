use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as create_associated_token_account, AssociatedToken, Create},
    token::{Mint, Token},
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
    /// CHECK: Proper PDA checks are made at the handler function
    #[account(
        mut,
        constraint = pull_request_token_account.to_account_info().data_is_empty() == true
    )]
    pub pull_request_token_account: UncheckedAccount<'info>,
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
    #[account(mut)]
    pub rewards_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<AddPullRequest>, metadata_uri: String) -> Result<()> {
    let pull_request_addr = &ctx.accounts.pull_request_addr;
    let issue = &ctx.accounts.issue;
    let commit = &ctx.accounts.commit;
    let pull_request_metadata_account = &mut ctx.accounts.pull_request_metadata_account;
    let rewards_mint = &ctx.accounts.rewards_mint;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let pull_request_token_account = &mut ctx.accounts.pull_request_token_account;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;

    msg!(
        "Adding pull request on issue {} by {}",
        issue.uri,
        pull_request_addr.key()
    );

    require!(issue.closed_at.is_none(), DefiOSError::IssueClosedAlready);

    create_associated_token_account(CpiContext::new(
        associated_token_program.to_account_info(),
        Create {
            payer: pull_request_addr.to_account_info(),
            associated_token: pull_request_token_account.to_account_info(),
            authority: pull_request_metadata_account.to_account_info(),
            mint: rewards_mint.to_account_info(),
            system_program: system_program.to_account_info(),
            token_program: token_program.to_account_info(),
        },
    ))?;

    pull_request_metadata_account.bump = *ctx.bumps.get("pull_request_metadata_account").unwrap();
    pull_request_metadata_account.sent_by = pull_request_addr.key();
    pull_request_metadata_account.commits = vec![commit.key()];
    pull_request_metadata_account.metadata_uri = metadata_uri.clone();
    pull_request_metadata_account.accepted = false;
    pull_request_metadata_account.pull_request_token_account = pull_request_token_account.key();

    emit!(PullRequestSent {
        sent_by: pull_request_addr.key(),
        commits: vec![commit.key()],
        metadata_uri: metadata_uri,
        issue: issue.key(),
        pull_request: pull_request_metadata_account.key()
    });

    Ok(())
}
