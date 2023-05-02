use anchor_lang::prelude::*;
use anchor_spl::token::{ transfer, Mint, Token, TokenAccount, Transfer };

use crate::{ error::DefiOSError, state::{ Issue, IssueStaker, Repository, IssueStaked} };

#[derive(Accounts)]
#[instruction(transfer_amount: u64)]
pub struct StakeIssue<'info> {
    #[account(mut)]
    pub issue_staker: Signer<'info>,
    #[account(
        mut,
        constraint = issue_staker_token_account.mint.eq(&issue_token_pool_account.mint),
        constraint = issue_staker_token_account.owner.eq(&issue_staker.key()),
        constraint = issue_staker_token_account.amount >= transfer_amount @ DefiOSError::InsufficientStakingFunds
    )]
    pub issue_staker_token_account: Account<'info, TokenAccount>,

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

    #[account(mut, address = issue_account.issue_token_pool_account)]
    pub issue_token_pool_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = issue_staker,
        space = IssueStaker::size(),
        seeds = [b"issuestaker", issue_account.key().as_ref(), issue_staker.key().as_ref()],
        bump
    )]
    pub issue_staker_account: Account<'info, IssueStaker>,

    #[account(constraint = rewards_mint.key().eq(&issue_token_pool_account.mint))]
    pub rewards_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<StakeIssue>, transfer_amount: u64) -> Result<()> {
    let issue_staker = &ctx.accounts.issue_staker;
    let issue_account = &ctx.accounts.issue_account;
    let issue_staker_account = &mut ctx.accounts.issue_staker_account;
    let issue_staker_token_account = &ctx.accounts.issue_staker_token_account;
    let issue_token_pool_account = &ctx.accounts.issue_token_pool_account;
    let rewards_mint = &ctx.accounts.rewards_mint;
    let staked_at = Clock::get()?.unix_timestamp;

    require!(issue_account.closed_at.is_none(), DefiOSError::IssueClosedAlready);

    msg!(
        "Staking {} including decimals of token {}",
        transfer_amount,
        rewards_mint.key().to_string()
    );

    transfer(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), Transfer {
            from: issue_staker_token_account.to_account_info(),
            to: issue_token_pool_account.to_account_info(),
            authority: issue_staker.to_account_info(),
        }),
        transfer_amount
    )?;

    issue_staker_account.bump = *ctx.bumps.get("issue_staker_account").unwrap();
    issue_staker_account.staked_amount = transfer_amount;
    issue_staker_account.staked_at = staked_at as u64;
    issue_staker_account.issue_staker = issue_staker.key();
    issue_staker_account.issue = issue_account.key();
    issue_staker_account.issue_staker_token_account = issue_token_pool_account.key();

    emit!(IssueStaked {
        issue_staker: issue_staker.key(),
        issue_account: issue_account.key(),
        staked_amount: transfer_amount,
        rewards_mint: rewards_mint.key(),
        issue_staker_token_account: issue_token_pool_account.key(),
        issue_contribution_link: issue_account.uri 
    });

    Ok(())
}