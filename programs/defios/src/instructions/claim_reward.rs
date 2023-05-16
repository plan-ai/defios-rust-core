use crate::{
    error::DefiOSError,
    state::{Issue, NameRouter, PullRequest, Repository, VerifiedUser},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{
        create as create_associated_token_account, get_associated_token_address, AssociatedToken,
        Create,
    },
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use sha256::digest;

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(
        seeds = [
            b"pullrequestadded",
            issue_account.key().as_ref(),
            pull_request_creator.key().as_ref()],
    bump = pull_request.bump)]
    pub pull_request: Account<'info, PullRequest>,
    #[account(
        mut,
        constraint = pull_request_creator.key().eq(&pull_request_verified_user.user_pubkey) @ DefiOSError::UnauthorizedUser,
    )]
    pub pull_request_creator: Signer<'info>,

    /// CHECK: PDA check is done at the handler function
    #[account(mut)]
    pub pull_request_creator_reward_account: UncheckedAccount<'info>,
    #[account(address = repository_account.rewards_mint)]
    pub rewards_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
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
    pub name_router_account: Box<Account<'info, NameRouter>>,

    #[account(
        seeds = [
            pull_request_verified_user.user_name.as_bytes(),
            pull_request_creator.key().as_ref(),
            name_router_account.key().as_ref(),
        ],
        bump = pull_request_verified_user.bump,
    )]
    pub pull_request_verified_user: Box<Account<'info, VerifiedUser>>,

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
    pub repository_account: Box<Account<'info, Repository>>,

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
    pub issue_account: Box<Account<'info, Issue>>,

    #[account(mut, address = issue_account.issue_token_pool_account)]
    pub issue_token_pool_account: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ClaimReward>) -> Result<()> {
    let pull_request_creator = &mut ctx.accounts.pull_request_creator;
    let rewards_mint = &ctx.accounts.rewards_mint;
    let repository_account = &ctx.accounts.repository_account;
    let issue_account = &mut ctx.accounts.issue_account;
    let pull_request_creator_reward_account = &mut ctx.accounts.pull_request_creator_reward_account;
    let issue_token_pool_account = &mut ctx.accounts.issue_token_pool_account;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;
    let pull_request = &ctx.accounts.pull_request;

    //checking if issue token account sent is same as expected
    let expected_issue_token_pool_account =
        get_associated_token_address(&issue_account.key(), &rewards_mint.key());

    require!(
        expected_issue_token_pool_account.eq(&issue_token_pool_account.key()),
        DefiOSError::TokenAccountMismatch
    );

    //require pull request to be accepted to claim rewards
    require!(
        pull_request.accepted,
        DefiOSError::PullRequestNotYetAccepted
    );

    //Creating token account if empty
    if pull_request_creator_reward_account.data_is_empty() {
        msg!("Creating Commit creator reward token account");
        create_associated_token_account(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: pull_request_creator.to_account_info(),
                associated_token: pull_request_creator_reward_account.to_account_info(),
                authority: pull_request_creator.to_account_info(),
                mint: rewards_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }

    // Transferring pool balance to commit creator
    let issue_index_str = repository_account.issue_index.to_string();
    let repository_account_key = repository_account.key();
    let issue_creator_key = issue_account.issue_creator.key();

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"issue",
        issue_index_str.as_bytes(),
        repository_account_key.as_ref(),
        issue_creator_key.as_ref(),
        &[issue_account.bump],
    ]];

    let token_balance = issue_token_pool_account.amount;

    require!(
        token_balance>0,
        DefiOSError::NoMoneyStakedOnIssue
    );
    
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: issue_token_pool_account.to_account_info(),
                to: pull_request_creator_reward_account.to_account_info(),
                authority: issue_account.to_account_info(),
            },
            signer_seeds,
        ),
        token_balance,
    )?;

    Ok(())
}
