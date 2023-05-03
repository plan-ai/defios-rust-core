use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{
        create as create_associated_token_account, get_associated_token_address, AssociatedToken,
        Create,
    },
    token::{Mint, Token},
};

use crate::{
    error::DefiOSError,
    state::{Issue, NameRouter, Repository, VerifiedUser, IssueCreated},
};

#[derive(Accounts)]
#[instruction(name: String)]
pub struct AddIssue<'info> {
    #[account(
        mut,
        address = issue_verified_user.user_pubkey @ DefiOSError::UnauthorizedUser,
    )]
    pub issue_creator: Signer<'info>,

    #[account(
        address = name_router_account.router_creator
    )]
    pub router_creator: SystemAccount<'info>,

    #[account(
        address = repository_account.repository_creator
    )]
    pub repository_creator: SystemAccount<'info>,

    #[account(
        seeds = [
            issue_verified_user.user_name.as_bytes(),
            issue_creator.key().as_ref(),
            name_router_account.key().as_ref()
        ],
        bump = issue_verified_user.bump
    )]
    pub issue_verified_user: Account<'info, VerifiedUser>,

    #[account(
        address = issue_verified_user.name_router @ DefiOSError::InvalidNameRouter,
        seeds = [
            name_router_account.signing_domain.as_bytes(),
            name_router_account.signature_version.to_string().as_bytes(),
            router_creator.key().as_ref()
        ],
        bump = name_router_account.bump
    )]
    pub name_router_account: Account<'info, NameRouter>,

    #[account(
        mut,
        seeds = [
            b"repository",
            repository_account.name.as_bytes(),
            repository_creator.key().as_ref(),
        ],
        bump = repository_account.bump
    )]
    pub repository_account: Account<'info, Repository>,

    #[account(
        init,
        space = Issue::size(),
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

    /// CHECK: Proper PDA checks are made at the handler function
    #[account(
        mut,
        constraint = issue_token_pool_account.to_account_info().data_is_empty() == true
    )]
    pub issue_token_pool_account: UncheckedAccount<'info>,

    #[account(
        address = repository_account.rewards_mint,
    )]
    pub rewards_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<AddIssue>, uri: String) -> Result<()> {
    let repository_account = &mut ctx.accounts.repository_account;
    let issue_account = &mut ctx.accounts.issue_account;
    let issue_token_pool_account = &mut ctx.accounts.issue_token_pool_account;
    let issue_verified_user = &ctx.accounts.issue_verified_user;
    let rewards_mint = &ctx.accounts.rewards_mint;
    let created_at = Clock::get()?.unix_timestamp;
    let mint = &ctx.accounts.rewards_mint.to_account_info();

    msg!(
        "Creating issue under repository: {} Issue address: {} Issue Token pool address {}",
        repository_account.key().to_string(),
        issue_account.key().to_string(),
        issue_token_pool_account.key().to_string(),
    );

    let expected_issue_token_pool_account =
        get_associated_token_address(&issue_account.key(), &rewards_mint.key());

    require!(
        expected_issue_token_pool_account.eq(&issue_token_pool_account.key()),
        DefiOSError::TokenAccountMismatch
    );

    create_associated_token_account(CpiContext::new(
        ctx.accounts.associated_token_program.to_account_info(),
        Create {
            payer: ctx.accounts.issue_creator.to_account_info(),
            associated_token: issue_token_pool_account.to_account_info(),
            authority: issue_account.to_account_info(),
            mint: ctx.accounts.rewards_mint.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        },
    ))?;

    issue_account.bump = *ctx.bumps.get("issue_account").unwrap();
    issue_account.index = repository_account.issue_index;
    issue_account.created_at = created_at as u64;
    issue_account.issue_creator = issue_verified_user.user_pubkey.key();
    issue_account.issue_token_pool_account = issue_token_pool_account.key();
    issue_account.commit_index = 0;
    issue_account.repository = repository_account.key();
    issue_account.uri = uri;
    issue_account.closed_at = None;

    repository_account.issue_index = repository_account.issue_index.saturating_add(1);

    emit!(IssueCreated {
        issue_creator: issue_verified_user.user_pubkey.key(),
        issue_account: issue_account.key(),
        issue_token_pool_account: issue_token_pool_account.key(),
        repository_account: repository_account.key(),
        uri: issue_account.uri.clone(),
        rewards_mint: mint.key(),
    });

    Ok(())
}
