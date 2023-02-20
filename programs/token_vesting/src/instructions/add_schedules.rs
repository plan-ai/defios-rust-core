use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::{
    error::TokenVestingError,
    state::{Schedule, VestingSchedule},
};

#[derive(Accounts)]
pub struct AddSchedules<'info> {
    #[account(
        address = vesting_account.authority
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        constraint = authority_token_account.mint.eq(&token_mint.key()),
        constraint = authority_token_account.owner.eq(&authority.key()),
        address = vesting_account.destination_address,
    )]
    pub authority_token_account: Account<'info, TokenAccount>,

    #[account(
        address = vesting_account.mint_address,
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = !vesting_account.is_initialized @ TokenVestingError::VestingAccountAlreadyInitialized,
        seeds = [
            b"vesting",
            token_mint.key().as_ref(),
            authority.key().as_ref(),
        ],
        bump = vesting_account.bump
    )]
    pub vesting_account: Account<'info, VestingSchedule>,

    #[account(
        mut,
        constraint = vesting_token_account.mint.eq(&token_mint.key()),
        constraint = vesting_token_account.owner.eq(&vesting_account.key()),
        constraint = vesting_token_account.close_authority.is_none() @ TokenVestingError::VestingAccountInvalidClose,
        constraint = vesting_token_account.delegate.is_none() @ TokenVestingError::VestingAccountInvalidDelegate,
    )]
    pub vesting_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<AddSchedules>, schedules: Vec<Schedule>) -> Result<()> {
    let vesting_account = &mut ctx.accounts.vesting_account;
    let authority = &ctx.accounts.authority;

    let token_program = &ctx.accounts.token_program;
    let authority_token_account = &ctx.accounts.authority_token_account;
    let vesting_token_account = &ctx.accounts.vesting_token_account;

    require!(
        schedules.len() as u64 != vesting_account.max_schedules,
        TokenVestingError::SchedulesLimitReached
    );

    let token_balance = authority_token_account.amount;
    let amount_to_transfer = schedules.iter().fold(0, |acc, schedule| {
        return acc + schedule.amount;
    });

    require!(
        token_balance >= amount_to_transfer,
        TokenVestingError::InsufficientFunds
    );

    for Schedule {
        release_time,
        amount,
    } in schedules.iter()
    {
        vesting_account.schedules.push(Schedule {
            release_time: *release_time,
            amount: *amount,
        });
    }

    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: authority_token_account.to_account_info(),
                to: vesting_token_account.to_account_info(),
                authority: authority.to_account_info(),
            },
        ),
        amount_to_transfer,
    )?;

    vesting_account.is_initialized = true;

    Ok(())
}
