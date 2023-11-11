use crate::{
    error::DefiOSError,
    event::IssueCreated,
    state::{Issue, Repository, VerifiedUser},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(name: String)]
pub struct AddIssue<'info> {
    #[account(
        mut,
        address = issue_verified_user.user_pubkey @ DefiOSError::UnauthorizedUser,
    )]
    pub issue_creator: Signer<'info>,
    #[account(
        seeds = [
            issue_verified_user.user_name.as_bytes(),
            issue_creator.key().as_ref(),
            issue_verified_user.name_router.key().as_ref()
        ],
        bump = issue_verified_user.bump
    )]
    pub issue_verified_user: Account<'info, VerifiedUser>,

    #[account(
        mut,
        seeds = [
            b"repository",
            repository_account.id.as_bytes(),
            repository_account.repository_creator.key().as_ref(),
        ],
        bump = repository_account.bump
    )]
    pub repository_account: Account<'info, Repository>,

    #[account(
        init,
        space = 8+Issue::INIT_SPACE,
        payer = issue_creator,
        seeds = [
            b"issue",
            repository_account.issue_index.to_string().as_bytes(),
            repository_account.key().as_ref(),
            issue_creator.key().as_ref(),
        ],
        bump
    )]
    pub issue_account: Account<'info, Issue>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddIssue>, uri: String) -> Result<()> {
    let repository_account = &mut ctx.accounts.repository_account;
    let issue_account = &mut ctx.accounts.issue_account;
    let issue_creator = &ctx.accounts.issue_creator;
    let created_at = Clock::get()?.unix_timestamp;

    issue_account.bump = *ctx.bumps.get("issue_account").unwrap();
    issue_account.index = repository_account.issue_index;
    issue_account.created_at = created_at as u64;
    issue_account.issue_creator = issue_creator.key();
    issue_account.commit_index = 0;
    issue_account.repository = repository_account.key();
    issue_account.uri = uri;
    issue_account.closed_at = None;
    issue_account.issue_token = repository_account.repo_token.unwrap();
    repository_account.issue_index = repository_account.issue_index.saturating_add(1);
    repository_account.num_open_issues += 1;

    emit!(IssueCreated {
        issue_creator: issue_creator.key(),
        issue_account: issue_account.key(),
        repository_account: repository_account.key(),
        uri: issue_account.uri.clone(),
    });

    Ok(())
}
