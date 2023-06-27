use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{
        create as create_associated_token_account, get_associated_token_address, AssociatedToken,
        Create,
    },
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use crate::error::DefiOSError;
use crate::event::PullRequestStaked;
use crate::state::{Issue, PRStaker, PullRequest};

#[derive(Accounts)]
#[instruction(transfer_amount:u64)]
pub struct StakePR<'info> {
    ///CHECK: Check done using other constraints
    #[account(mut, address = pull_request_metadata_account.sent_by)]
    pub pull_request_addr: AccountInfo<'info>,
    #[account(mut)]
    pub issue: Account<'info, Issue>,
    #[account(
        seeds = [
            b"pullrequestadded",
            issue.key().as_ref(),
            pull_request_addr.key().as_ref()
        ],
        bump
    )]
    pub pull_request_metadata_account: Account<'info, PullRequest>,
    ///CHECK: Handling of account is done in function
    #[account(mut)]
    pub pull_request_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub pull_request_staker: Signer<'info>,
    #[account(
        mut,
        constraint = pull_request_staker_token_account.owner.eq(&pull_request_staker.key()),
        constraint = pull_request_staker_token_account.amount >= transfer_amount @ DefiOSError::InsufficientStakingFunds
    )]
    pub pull_request_staker_token_account: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = pull_request_staker,
        space = 8+PRStaker::INIT_SPACE,
        seeds = [b"pullrestaker", pull_request_metadata_account.key().as_ref(), pull_request_staker.key().as_ref()],
        bump
    )]
    pub pull_request_staker_account: Account<'info, PRStaker>,
    #[account(mut)]
    pub rewards_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<StakePR>, transfer_amount: u64) -> Result<()> {
    let pull_request_staker = &ctx.accounts.pull_request_staker;
    let pull_request_metadata_account = &ctx.accounts.pull_request_metadata_account;
    let pull_request_staker_account = &mut ctx.accounts.pull_request_staker_account;
    let pull_request_staker_token_account = &ctx.accounts.pull_request_staker_token_account;
    let pull_request_token_account = &ctx.accounts.pull_request_token_account;
    let rewards_mint = &ctx.accounts.rewards_mint;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let staked_at = Clock::get()?.unix_timestamp;
    let issue = &ctx.accounts.issue;
    require!(issue.closed_at.is_none(), DefiOSError::IssueClosedAlready);

    //Creating token account if empty
    if pull_request_token_account.data_is_empty() {
        create_associated_token_account(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: pull_request_staker.to_account_info(),
                associated_token: pull_request_token_account.to_account_info(),
                authority: pull_request_metadata_account.to_account_info(),
                mint: rewards_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }

    //checks coorect mint accounts sent
    let expected_pull_reuquest_token_account =
        get_associated_token_address(&pull_request_metadata_account.key(), &rewards_mint.key());

    let expected_staker_token_account =
        get_associated_token_address(&pull_request_staker.key(), &rewards_mint.key());

    require!(
        expected_staker_token_account.eq(&pull_request_staker_token_account.key())
            & expected_pull_reuquest_token_account.eq(&pull_request_token_account.key()),
        DefiOSError::TokenAccountMismatch
    );

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: pull_request_staker_token_account.to_account_info(),
                to: pull_request_token_account.to_account_info(),
                authority: pull_request_staker.to_account_info(),
            },
        ),
        transfer_amount,
    )?;

    pull_request_staker_account.bump = *ctx.bumps.get("pull_request_staker_account").unwrap();
    pull_request_staker_account.staked_amount += transfer_amount;
    pull_request_staker_account.staked_at.push(staked_at as u64);
    pull_request_staker_account.pr_staker = pull_request_staker.key();
    pull_request_staker_account.pr = pull_request_metadata_account.key();
    pull_request_staker_account.pr_staker_token_account = pull_request_staker_token_account.key();

    emit!(PullRequestStaked {
        pr_staker: pull_request_staker.key(),
        pr_staker_token_account: pull_request_staker_token_account.key(),
        pr_account: pull_request_metadata_account.key(),
        staked_amount: transfer_amount,
        rewards_mint: rewards_mint.key(),
        pr_contribution_link: pull_request_metadata_account.metadata_uri.clone()
    });

    Ok(())
}
