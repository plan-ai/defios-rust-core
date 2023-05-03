use crate::{
    error::DefiOSError,
    state::{Commit, Issue, NameRouter, Repository, VerifiedUser, CommitAdded},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(commit_hash: String)]
pub struct AddCommit<'info> {
    #[account(
        address = name_router_account.router_creator @ DefiOSError::UnauthorizedUser,
    )]
    pub router_creator: SystemAccount<'info>,

    #[account(
        seeds = [
            name_router_account.signing_domain.as_bytes(),
            name_router_account.signature_version.to_string().as_bytes(),
            router_creator.key().as_ref()
        ],
        bump = name_router_account.bump
    )]
    pub name_router_account: Account<'info, NameRouter>,

    #[account(
        address = repository_account.repository_creator
    )]
    pub repository_creator: SystemAccount<'info>,

    #[account(
        address = issue_account.issue_creator
    )]
    pub issue_creator: SystemAccount<'info>,

    #[account(
        seeds = [
            b"repository",
            repository_account.name.as_bytes(),
            repository_creator.key().as_ref(),
        ],
        bump = repository_account.bump
    )]
    pub repository_account: Account<'info, Repository>,

    #[account(
        mut,
        seeds = [
            b"issue",
            issue_account.index.to_string().as_bytes(),
            repository_account.key().as_ref(),
            issue_creator.key().as_ref(),
        ],
        bump = issue_account.bump
    )]
    pub issue_account: Account<'info, Issue>,

    #[account(
        mut,
        constraint = commit_creator.key().eq(&commit_verified_user.user_pubkey) @ DefiOSError::UnauthorizedUser,
    )]
    pub commit_creator: Signer<'info>,

    #[account(
        seeds = [
            commit_verified_user.user_name.as_bytes(),
            commit_creator.key().as_ref(),
            name_router_account.key().as_ref(),
        ],
        bump = commit_verified_user.bump,
    )]
    pub commit_verified_user: Account<'info, VerifiedUser>,

    #[account(
        init,
        payer = commit_creator,
        space = Commit::size(),
        seeds = [
            b"commit",
            commit_hash.as_bytes(),
            commit_creator.key().as_ref(),
            issue_account.key().as_ref(),
        ],
        bump
    )]
    pub commit_account: Account<'info, Commit>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AddCommit>,
    commit_hash: String,
    tree_hash: String,
    metadata_uri: String,
) -> Result<()> {
    let issue_account = &mut ctx.accounts.issue_account;
    let commit_creator = &ctx.accounts.commit_creator;
    let commit_account = &mut ctx.accounts.commit_account;
    let created_at = Clock::get()?.unix_timestamp;

    msg!(
        "Adding commit under issue: {} Commit state address: {} Commit creator: {}",
        issue_account.key().to_string(),
        commit_account.key().to_string(),
        commit_creator.key().to_string()
    );
    let metadata = metadata_uri.clone();
    commit_account.bump = *ctx.bumps.get("commit_account").unwrap();
    commit_account.index = issue_account.commit_index;
    commit_account.tree_hash = tree_hash;
    commit_account.commit_hash = commit_hash;
    commit_account.metadata_uri = metadata_uri;
    commit_account.created_at = created_at as u64;
    commit_account.commit_creator = ctx.accounts.commit_creator.key();

    issue_account.commit_index = issue_account.commit_index.saturating_add(1);

    emit!(CommitAdded {
        commit_creator: commit_creator.key(),
        commit_account: commit_account.key(),
        issue_account: issue_account.key(),
        metadata_uri: metadata,
    });

    Ok(())
}