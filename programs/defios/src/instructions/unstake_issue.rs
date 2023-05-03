use anchor_lang::prelude::*;
use anchor_spl::token::{
    close_account, transfer, CloseAccount, Mint, Token, TokenAccount, Transfer,
};

use crate::{
    error::DefiOSError,
    state::{Issue, IssueStaker, Repository, IssueUnstaked},
};

#[derive(Accounts)]
#[instruction()]
pub struct UnstakeIssue<'info> {
    #[account(mut)]
    pub issue_staker: Signer<'info>,

    #[account(
        mut,
        constraint = issue_staker_token_account.mint.eq(&issue_token_pool_account.mint),
        constraint = issue_staker_token_account.owner.eq(&issue_staker.key()),
    )]
    pub issue_staker_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = issue_account.repository,
        seeds = [
            b"repository",
            repository_account.name.as_bytes(),
            repository_account.repository_creator.key().as_ref(),
        ],
        bump = repository_account.bump
    )]
    pub repository_account: Box<Account<'info, Repository>>,

    #[account(
        seeds = [
            b"issue",
            issue_account.index.to_string().as_bytes(),
            repository_account.key().as_ref(),
            issue_account.issue_creator.key().as_ref(),
        ],
        bump = issue_account.bump
    )]
    pub issue_account: Box<Account<'info, Issue>>,

    #[account(
        mut,
        address = issue_account.issue_token_pool_account,
        constraint = issue_token_pool_account.amount >= issue_staker_account.staked_amount @ DefiOSError::InsufficientStakingFunds
    )]
    pub issue_token_pool_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        close = issue_staker,
        seeds = [
            b"issuestaker",
            issue_account.key().as_ref(),
            issue_staker.key().as_ref()
        ],
        bump = issue_staker_account.bump
    )]
    pub issue_staker_account: Box<Account<'info, IssueStaker>>,

    #[account(
        constraint = rewards_mint.key().eq(&issue_token_pool_account.mint)
    )]
    pub rewards_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<UnstakeIssue>) -> Result<()> {
    let issue_staker = &ctx.accounts.issue_staker;
    let issue_account = &ctx.accounts.issue_account;
    let repository_account = &ctx.accounts.repository_account;
    let issue_staker_account = &mut ctx.accounts.issue_staker_account;
    let issue_staker_token_account = &ctx.accounts.issue_staker_token_account;
    let issue_token_pool_account = &ctx.accounts.issue_token_pool_account;
    let rewards_mint = &ctx.accounts.rewards_mint;

    require!(
        issue_account.closed_at.is_none(),
        DefiOSError::IssueClosedAlready
    );

    msg!(
        "Unstaking {} including decimals of token {}",
        issue_staker_account.staked_amount,
        rewards_mint.key().to_string()
    );

    let issue_index_str = issue_account.index.to_string();
    let repository_account_key = repository_account.key();
    let issue_creator_key = issue_account.issue_creator.key();

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"issue",
        issue_index_str.as_bytes(),
        repository_account_key.as_ref(),
        issue_creator_key.as_ref(),
        &[issue_account.bump],
    ]];

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: issue_token_pool_account.to_account_info(),
                to: issue_staker_token_account.to_account_info(),
                authority: issue_account.to_account_info(),
            },
            signer_seeds,
        ),
        issue_staker_account.staked_amount,
    )?;

    close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: issue_token_pool_account.to_account_info(),
            authority: issue_account.to_account_info(),
            destination: issue_staker.to_account_info(),
        },
        signer_seeds,
    ))?;

    emit!(IssueUnstaked {
        issue_account: issue_account.key(),
        issue_staker: issue_staker.key(),
        issue_staker_token_account: issue_staker_token_account.key(),
        rewards_mint: rewards_mint.key(),
        unstaked_amount: issue_staker_account.staked_amount,
        issue_contribution_link: issue_account.uri
    });

    Ok(())
}