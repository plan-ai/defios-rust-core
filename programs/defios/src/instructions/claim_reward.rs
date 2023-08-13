use crate::{
    error::DefiOSError,
    state::{Issue, PullRequest, Repository},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{
        create as create_associated_token_account, get_associated_token_address, AssociatedToken,
        Create,
    },
    mint::USDC,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

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
        constraint = pull_request_creator.key().eq(&pull_request.sent_by) @ DefiOSError::UnauthorizedUser,
    )]
    pub pull_request_creator: Signer<'info>,

    /// CHECK: PDA check is done at the handler function
    #[account(mut)]
    pub pull_request_creator_reward_account: UncheckedAccount<'info>,
    #[account(
        constraint = rewards_mint.key()==repository_account.rewards_mint || rewards_mint.key() == USDC,
        constraint = rewards_mint.key().eq(&issue_token_pool_account.mint)
    )]
    pub rewards_mint: Account<'info, Mint>,
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
            repository_account.id.as_bytes(),
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
    pub issue_account: Box<Account<'info, Issue>>,

    #[account(mut)]
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
    let pull_request = &mut ctx.accounts.pull_request;

    //Creating token account if empty
    if pull_request_creator_reward_account.data_is_empty() {
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

    //checking if issue token account sent is same as expected
    let expected_issue_token_pool_account =
        get_associated_token_address(&issue_account.key(), &rewards_mint.key());

    let expected_pull_request_creator_reward_account =
        get_associated_token_address(&pull_request_creator.key(), &rewards_mint.key());

    require!(
        expected_issue_token_pool_account.eq(&issue_token_pool_account.key())
            && expected_pull_request_creator_reward_account
                .eq(&pull_request_creator_reward_account.key()),
        DefiOSError::TokenAccountMismatch
    );

    //require pull request to be accepted to claim rewards
    require!(
        pull_request.accepted,
        DefiOSError::PullRequestNotYetAccepted
    );

    // Transferring pool balance to commit creator
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

    let issue_token_balance = issue_token_pool_account.amount;
    require!(issue_token_balance > 0, DefiOSError::NoMoneyStakedOnIssue);

    if issue_token_balance > 0 {
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
            issue_token_balance,
        )?;
    };

    Ok(())
}
