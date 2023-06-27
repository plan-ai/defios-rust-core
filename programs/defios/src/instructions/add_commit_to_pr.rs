use crate::error::DefiOSError;
use crate::event::AddCommitToPR;
use crate::state::{Commit, PullRequest};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AddCommitToPullRequest<'info> {
    #[account(mut)]
    pub commit_addr: Signer<'info>,
    #[account(mut)]
    pub pull_request_metadata_account: Account<'info, PullRequest>,
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
