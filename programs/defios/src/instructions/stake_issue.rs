use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{
        create as create_associated_token_account, get_associated_token_address, AssociatedToken,
        Create,
    },
    mint::USDC,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::{
    error::DefiOSError,
    event::IssueStaked,
    helper::{calculate_sell_amount, find_index},
    state::{Issue, IssueStaker, PullRequest, Repository},
};

#[derive(Accounts)]
#[instruction(transfer_amount: u64)]
#[event_cpi]
pub struct StakeIssue<'info> {
    #[account(mut)]
    pub issue_staker: Signer<'info>,
    #[account(
        mut,
        constraint = issue_staker_token_account.owner.eq(&issue_staker.key()),
        constraint = issue_staker_token_account.amount >= transfer_amount @ DefiOSError::InsufficientStakingFunds
    )]
    pub issue_staker_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = issue_account.repository,
        seeds = [
            b"repository",
            repository_account.id.as_bytes(),
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
    pub issue_account: Account<'info, Issue>,
    ///CHECK: Handling of account is done in function
    #[account(mut)]
    pub issue_token_pool_account: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = issue_staker,
        space = 8+IssueStaker::INIT_SPACE,
        seeds = [b"issuestaker", issue_account.key().as_ref(), issue_staker.key().as_ref()],
        bump
    )]
    pub issue_staker_account: Account<'info, IssueStaker>,

    #[account(mut,constraint = rewards_mint.key()==repository_account.rewards_mint || rewards_mint.key() == USDC)]
    pub rewards_mint: Account<'info, Mint>,
    #[account(
        seeds = [
            b"pullrequestadded",
            issue_account.key().as_ref(),
            pull_request_metadata_account.sent_by.key().as_ref()
        ],
        bump=pull_request_metadata_account.bump
    )]
    pub pull_request_metadata_account: Option<Account<'info, PullRequest>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let staked_at = Clock::get()?.unix_timestamp;
    let pull_request_metadata_account = &mut ctx.accounts.pull_request_metadata_account;

    require!(
        issue_account.closed_at.is_none(),
        DefiOSError::IssueClosedAlready
    );

    //Creating token account if empty
    if issue_token_pool_account.data_is_empty() {
        create_associated_token_account(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: issue_staker.to_account_info(),
                associated_token: issue_token_pool_account.to_account_info(),
                authority: issue_account.to_account_info(),
                mint: rewards_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }

    //checks coorect mint accounts sent
    let expected_issue_token_pool_account =
        get_associated_token_address(&issue_account.key(), &rewards_mint.key());

    let expected_issue_staker_token_account =
        get_associated_token_address(&issue_staker.key(), &rewards_mint.key());
    require!(
        expected_issue_token_pool_account.eq(&issue_token_pool_account.key())
            & expected_issue_staker_token_account.eq(&issue_staker_token_account.key()),
        DefiOSError::TokenAccountMismatch
    );

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: issue_staker_token_account.to_account_info(),
                to: issue_token_pool_account.to_account_info(),
                authority: issue_staker.to_account_info(),
            },
        ),
        transfer_amount,
    )?;

    if let Some(index) = find_index(
        &issue_staker_account.issue_staker_token_account,
        &issue_token_pool_account.key(),
    ) {
        issue_staker_account.staked_amount[index] += transfer_amount;
    } else {
        issue_staker_account.staked_amount.push(transfer_amount);
        issue_staker_account
            .issue_staker_token_account
            .push(issue_token_pool_account.key())
    }

    issue_staker_account.bump = *ctx.bumps.get("issue_staker_account").unwrap();
    issue_staker_account.issue_staker = issue_staker.key();
    issue_staker_account.issue = issue_account.key();
    let voting_power: u64;
    if rewards_mint.key() == USDC {
        voting_power = transfer_amount as u64
    } else {
        voting_power = calculate_sell_amount(rewards_mint.supply, transfer_amount as u64) as u64;
    };

    if issue_staker_account.has_voted == false {
        issue_staker_account.pr_voting_power += voting_power
    } else {
        let voted_on: Option<Pubkey> = issue_staker_account.voted_on;
        if let (Some(pull_request_metadata_account), Some(voted_on)) =
            (pull_request_metadata_account, voted_on)
        {
            require!(
                voted_on == pull_request_metadata_account.key(),
                DefiOSError::PullRequestAutoUpdate
            );
            pull_request_metadata_account.total_voted_amount += voting_power;
        } else {
            require!(1 == 0, DefiOSError::PullRequestAutoUpdate);
        }
    };

    issue_staker_account.issue_unstakable = true;

    emit_cpi!(IssueStaked {
        issue_staker: issue_staker.key(),
        issue_account: issue_account.key(),
        staked_amount: transfer_amount,
        rewards_mint: rewards_mint.key(),
        issue_staker_token_account: issue_token_pool_account.key(),
        issue_contribution_link: issue_account.uri.clone(),
        staked_at: staked_at,
        pr_voting_power: voting_power
    });

    Ok(())
}
