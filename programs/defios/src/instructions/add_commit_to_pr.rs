use crate::error::DefiOSError;
use crate::state::{AddCommitToPR, Commit, NameRouter, PullRequest, VerifiedUser};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AddCommitToPullRequest<'info> {
    #[account(mut)]
    pub commit_addr: Signer<'info>,
    #[account(mut)]
    pub pull_request_metadata_account: Account<'info, PullRequest>,
    #[account(
        seeds = [
            commit_verified_user.user_name.as_bytes(),
            commit_addr.key().as_ref(),
            name_router_account.key().as_ref()
        ],
        bump = commit_verified_user.bump
    )]
    pub commit_verified_user: Account<'info, VerifiedUser>,

    #[account(
        address = commit_verified_user.name_router @ DefiOSError::InvalidNameRouter,
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

pub fn handler(ctx: Context<AddCommitToPullRequest>) -> Result<()> {
    let commit_addr = &ctx.accounts.commit_addr;
    let pull_request_metadata_account = &mut ctx.accounts.pull_request_metadata_account;

    let mut commit: Account<Commit>;
    for account in ctx.remaining_accounts.to_vec().iter() {
        commit = Account::try_from(account)?;
        require!(
            pull_request_metadata_account
                .sent_by
                .eq(&commit.commit_creator)
                & commit_addr.key().eq(&commit.commit_creator),
            DefiOSError::UnauthorizedPR
        );
        pull_request_metadata_account.commits.push(commit.key());
    }

    emit!(AddCommitToPR {
        commit: pull_request_metadata_account.commits.clone(),
        by: commit_addr.key()
    });

    Ok(())
}
