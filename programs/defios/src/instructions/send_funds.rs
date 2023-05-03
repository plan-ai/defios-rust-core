use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};

use crate::{
    error::DefiOSError,
    state::{Objective, StakeOnObjectiveEvent},
    ObjectiveState::InProgress,
};

#[derive(Accounts)]
#[instruction(transfer_amount: u64)]
pub struct StakeObjective<'info> {
    #[account(mut)]
    pub objective_staker: Signer<'info>,
    #[account(
        mut,
        constraint = objective_staker_token_account.amount >= transfer_amount @ DefiOSError::InsufficientStakingFunds
    )]
    pub objective_staker_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub objective_account: Account<'info, Objective>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<StakeObjective>, transfer_amount: u64) -> Result<()> {
    let objective_staker = &mut ctx.accounts.objective_staker;
    let objective_staker_token_account = &mut ctx.accounts.objective_staker_token_account;
    let objective_account = &mut ctx.accounts.objective_account;

    msg!(
        "Sending Funds: From:{}, To: {}",
        objective_staker.key(),
        objective_account.key()
    );

    require!(
        objective_account.objective_state == InProgress,
        DefiOSError::ObjectiveClosedAlready
    );

    let mut ispresent = false;

    // to do, optimise for loops
    for (i, staker) in objective_account.objective_staker_ids.iter().enumerate() {
        if *staker == objective_staker.key() {
            objective_account.objective_staker_amts[i] += transfer_amount;
            ispresent = true;
            break;
        }
    }

    if ispresent {
        objective_account
            .objective_staker_ids
            .push(objective_staker.key());
        objective_account
            .objective_staker_amts
            .push(transfer_amount);
    }

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: objective_staker_token_account.to_account_info(),
                to: objective_account.to_account_info(),
                authority: objective_staker.to_account_info(),
            },
        ),
        transfer_amount,
    )?;

    emit!(StakeOnObjectiveEvent {
        objective_pub_key: objective_account.key(),
        staked_by: objective_staker.key()
    });
    Ok(())
}