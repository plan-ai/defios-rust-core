use crate::error::DefiOSError;
use crate::event::GrantDispersed;
use crate::state::{Issue, Objective, Repository};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as create_associated_token_account, AssociatedToken, Create},
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(disperse_amount: u64)]
pub struct DisperseGrant<'info> {
    #[account(mut,constraint = repository.repository_creator == repository_creator.key())]
    pub repository_creator: Signer<'info>,
    #[account(
        mut,
        constraint = objective.objective_repository == repository.key()
    )]
    pub objective: Account<'info, Objective>,
    pub repository: Account<'info, Repository>,
    #[account(
        mut,
        seeds = [
            b"issue",
            issue_account.index.to_string().as_bytes(),
            repository.key().as_ref(),
            issue_account.issue_creator.key().as_ref(),
        ],
        bump = issue_account.bump
    )]
    pub issue_account: Box<Account<'info, Issue>>,
    ///CHECK: Handling of account is done in function
    #[account(mut)]
    pub issue_token_pool_account: UncheckedAccount<'info>,
    #[account(constraint = token_mint.key() == issue_account.issue_token)]
    pub token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = objective_stake_account.owner.eq(&objective.key()),
        constraint = objective_stake_account.amount >= disperse_amount @ DefiOSError::InsufficientStakingFunds,
        constraint = objective_stake_account.mint == token_mint.key()
    )]
    pub objective_stake_account: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<DisperseGrant>, disperse_amount: u64) -> Result<()> {
    let associated_token_program = &ctx.accounts.associated_token_program;
    let repository_creator = &mut ctx.accounts.repository_creator;
    let token_program = &ctx.accounts.token_program;
    let objective_stake_account = &mut ctx.accounts.objective_stake_account;
    let objective = &mut ctx.accounts.objective;
    let issue_account = &ctx.accounts.issue_account;
    let system_program = &ctx.accounts.system_program;
    let token_mint = &ctx.accounts.token_mint;
    let issue_token_pool_account = &mut ctx.accounts.issue_token_pool_account;

    //Creating token account if empty
    if issue_token_pool_account.data_is_empty() {
        create_associated_token_account(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: repository_creator.to_account_info(),
                associated_token: issue_token_pool_account.to_account_info(),
                authority: issue_account.to_account_info(),
                mint: token_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"objectivedataadd",
        objective.objective_creator_id.as_ref(),
        objective.objective_id.as_bytes(),
        &[objective.bump],
    ]];

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: objective_stake_account.to_account_info(),
                to: issue_token_pool_account.to_account_info(),
                authority: objective.to_account_info(),
            },
            signer_seeds,
        ),
        disperse_amount,
    )?;

    objective.total_dispersed_grant += disperse_amount;

    emit!(GrantDispersed {
        objective: objective.key(),
        issue: issue_account.key(),
        grant_amount: disperse_amount
    });

    Ok(())
}
