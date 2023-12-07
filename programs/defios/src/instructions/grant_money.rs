use crate::constants::TRUSTED_NAME_ROUTERS;
use crate::error::DefiOSError;
use crate::event::GrantProvided;
use crate::state::{Grantee, Objective, Repository, VerifiedUser};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as create_associated_token_account, AssociatedToken, Create},
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(transfer_amount: u64)]
pub struct GrantMoney<'info> {
    #[account(mut)]
    pub grantee: Signer<'info>,
    #[account(
        // constraint = TRUSTED_NAME_ROUTERS.contains(&grantee_verified_user.name_router),
        seeds = [
            grantee_verified_user.user_name.as_bytes(),
            grantee.key().as_ref(),
            grantee_verified_user.name_router.as_ref()
        ],
        bump = grantee_verified_user.bump
    )]
    pub grantee_verified_user: Box<Account<'info, VerifiedUser>>,
    #[account(
        mut,
        constraint = objective.objective_repository == repository.key()
    )]
    pub objective: Account<'info, Objective>,
    pub repository: Box<Account<'info, Repository>>,
    #[account(constraint = token_mint.key() == repository.repo_token)]
    pub token_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = grantee,
        space = 8 + Grantee::INIT_SPACE,
        seeds = [
            grantee.key().as_ref(),
            repository.key().as_ref(),
            objective.key().as_ref(),
        ],
        bump
    )]
    pub grantee_account: Account<'info, Grantee>,
    ///CHECK: The account checks are done in function, unchecked as it might not exist and will be created in that case
    #[account(mut)]
    pub objective_stake_account: AccountInfo<'info>,
    #[account(
        mut,
        constraint = grantee_stake_account.owner.eq(&grantee.key()),
        constraint = grantee_stake_account.amount >= transfer_amount @ DefiOSError::InsufficientStakingFunds,
        constraint = grantee_stake_account.mint == token_mint.key()
    )]
    pub grantee_stake_account: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<GrantMoney>,
    transfer_amount: u64,
    grant_metadata_uri: String,
) -> Result<()> {
    let grantee_account = &mut ctx.accounts.grantee_account;
    let grantee = &ctx.accounts.grantee;
    let grantee_stake_account = &ctx.accounts.grantee_stake_account;
    let objective = &mut ctx.accounts.objective;
    let objective_stake_account = &mut ctx.accounts.objective_stake_account;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;
    let token_mint = &ctx.accounts.token_mint;

    grantee_account.bump = ctx.bumps.grantee_account;
    grantee_account.grantee = grantee.key();
    grantee_account.objective = objective.key();
    grantee_account.staked_amount += transfer_amount;
    grantee_account.grant_metadata_uri = grant_metadata_uri.clone();
    objective.total_grant += transfer_amount;

    //Creating token account if empty
    if objective_stake_account.data_is_empty() {
        create_associated_token_account(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: grantee.to_account_info(),
                associated_token: objective_stake_account.to_account_info(),
                authority: objective.to_account_info(),
                mint: token_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }

    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: grantee_stake_account.to_account_info(),
                to: objective_stake_account.to_account_info(),
                authority: grantee.to_account_info(),
            },
        ),
        transfer_amount,
    )?;

    emit!(GrantProvided {
        grantee: grantee.key(),
        grant_amount: transfer_amount,
        objective: objective.key(),
        grant_metadata_uri: grant_metadata_uri
    });

    Ok(())
}
