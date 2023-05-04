use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as get_associated_token_address, AssociatedToken},
    token::Token,
};

use crate::state::{PullRequest,Commit,AddCommitToPR};

#[derive(Accounts)]
pub struct AddCommitToPullRequest<'info> {
    #[account(mut)]
    pub commit_addr: Signer<'info>,
    #[account(constraint = commit_addr.key() == commit.commit_creator)]
    pub commit: Account<'info,Commit>,
    #[account(mut)]
    pub pull_request_meatdata_account: Account<'info, PullRequest>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AddCommitToPullRequest>
) -> Result<()> {
    let commit_addr = &ctx.accounts.commit_addr;
    let commit = &ctx.accounts.commit;
    let pull_request_meatdata_account = &mut ctx.accounts.pull_request_meatdata_account;
    
    msg!(
        "Adding commit {} to pull request {}",
        commit.key(),
        pull_request_meatdata_account.key()
    );

    pull_request_meatdata_account.sent_by.push(commit_addr.key());
    pull_request_meatdata_account.commits.push(commit.key());

    emit!(AddCommitToPR {
        commit:commit.key(),
        by:commit_addr.key()
    });

    Ok(())
}