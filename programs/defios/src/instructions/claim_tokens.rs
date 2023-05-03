use crate::{
    error::DefiOSError,
    state::{NameRouter, Repository, UserClaim, VerifiedUser},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as create_associated_token_account, AssociatedToken, Create},
    token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
#[instruction(user_name: String)]
pub struct ClaimUserTokens<'info> {
    #[account(
        mut,
        constraint = user.key().eq(&verified_user.user_pubkey) @ DefiOSError::UnauthorizedUser,
    )]
    pub user: Signer<'info>,

    /// CHECK: PDA check is done at the handler function
    #[account(mut)]
    pub user_reward_token_account: UncheckedAccount<'info>,

    #[account(
        seeds = [
            verified_user.user_name.as_bytes(),
            user.key().as_ref(),
            name_router_account.key().as_ref(),
        ],
        bump = verified_user.bump,
    )]
    pub verified_user: Box<Account<'info, VerifiedUser>>,

    #[account(
        address = name_router_account.router_creator
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
    pub name_router_account: Account<'info, NameRouter>,

    #[account(
        mut,
        seeds = [
            b"user_claim",
            user_name.as_bytes(),
            repository_account.key().as_ref(),
            name_router_account.key().as_ref(),
        ],
        bump = user_claim_account.bump,
    )]
    pub user_claim_account: Box<Account<'info, UserClaim>>,

    #[account(address = repository_account.rewards_mint)]
    pub rewards_mint: Box<Account<'info, Mint>>,

    #[account(
        address = repository_account.repository_creator
    )]
    pub repository_creator: SystemAccount<'info>,

    #[account(
        seeds = [
            b"repository",
            repository_account.name.as_bytes(),
            repository_creator.key().as_ref(),
        ],
        bump = repository_account.bump
    )]
    pub repository_account: Account<'info, Repository>,

    #[account(mut, address = repository_account.repository_token_pool_account)]
    pub repository_token_pool_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<ClaimUserTokens>, _user_name: String) -> Result<()> {
    let user_claim_account = &mut ctx.accounts.user_claim_account;
    let repository_account = &ctx.accounts.repository_account;
    let user_reward_token_account = &mut ctx.accounts.user_reward_token_account;
    let user = &ctx.accounts.user;
    let rewards_mint = &ctx.accounts.rewards_mint;

    require!(!user_claim_account.is_claimed, DefiOSError::AlreadyClaimed);

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"repository",
        repository_account.name.as_bytes(),
        repository_account.repository_creator.as_ref(),
        &[repository_account.bump],
    ]];

    if user_reward_token_account.data_is_empty() {
        msg!("Creating Commit creator reward token account");
        create_associated_token_account(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            Create {
                payer: user.to_account_info(),
                associated_token: user_reward_token_account.to_account_info(),
                authority: user.to_account_info(),
                mint: rewards_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
        ))?;
    }

    let token_balance = user_claim_account.token_amount;
    let transfer_instruction = anchor_spl::token::Transfer {
        from: ctx.accounts.repository_token_pool_account.to_account_info(),
        to: ctx.accounts.user_reward_token_account.to_account_info(),
        authority: ctx.accounts.repository_account.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_instruction,
        signer_seeds,
    );
    anchor_spl::token::transfer(cpi_ctx, token_balance)?;

    // transfer(
    //     CpiContext::new_with_signer(
    //         ctx.accounts.token_program.to_account_info(),
    //         Transfer {
    //             from: ctx.accounts.repository_token_pool_account.to_account_info(),
    //             to: ctx.accounts.user_reward_token_account.to_account_info(),
    //             authority: ctx.accounts.repository_account.to_account_info(),
    //         },
    //         signer_seeds,
    //     ),
    //     token_balance,
    // )?;

    user_claim_account.is_claimed = true;

    Ok(())
}