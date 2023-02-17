use crate::{
    error::DefiOSError,
    state::{NameRouter, Repository, RepositoryCreated, VerifiedUser},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateRepository<'info> {
    #[account(
        mut,
        address = repository_verified_user.user_pubkey @ DefiOSError::UnauthorizedUser,
    )]
    pub repository_creator: Signer<'info>,

    #[account(
        seeds = [
            repository_verified_user.user_name.as_bytes(),
            repository_creator.key().as_ref(),
            name_router_account.key().as_ref()
        ],
        bump = repository_verified_user.bump
    )]
    pub repository_verified_user: Account<'info, VerifiedUser>,

    #[account(
        address = repository_verified_user.name_router @ DefiOSError::InvalidNameRouter,
        seeds = [
            name_router_account.signing_domain.as_bytes(),
            name_router_account.signature_version.to_string().as_bytes(),
            router_creator.key().as_ref()
        ],
        bump = name_router_account.bump
    )]
    pub name_router_account: Account<'info, NameRouter>,

    #[account(
        address = name_router_account.router_creator
    )]
    pub router_creator: SystemAccount<'info>,

    #[account(
        init,
        space = Repository::size(),
        payer = repository_creator,
        seeds = [
            b"repository",
            name.as_bytes(),
            repository_creator.key().as_ref(),
        ],
        bump
    )]
    pub repository_account: Account<'info, Repository>,

    pub repository_token_pool_account: Box<Account<'info, TokenAccount>>,
    pub rewards_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateRepository>,
    name: String,
    description: String,
    uri: String,
    gh_usernames: Vec<String>,
    claim_amounts: Vec<u64>,
) -> Result<()> {
    let repository_account = &mut ctx.accounts.repository_account;
    let name_router_account = &ctx.accounts.name_router_account;
    let repository_verified_user = &ctx.accounts.repository_verified_user;
    let rewards_mint_account = &ctx.accounts.rewards_mint;

    msg!(
        "Creating repository of name: {} Repository address: {} Rewards mint: {}",
        &name,
        repository_account.key().to_string(),
        rewards_mint_account.key().to_string(),
    );

    // TODO: Match URI username with verified user's username

    repository_account.bump = *ctx.bumps.get("repository_account").unwrap();
    repository_account.name_router = name_router_account.key();
    repository_account.repository_creator = repository_verified_user.user_pubkey.key();
    repository_account.rewards_mint = rewards_mint_account.key();
    repository_account.name = name;
    repository_account.description = description;
    repository_account.uri = uri;
    repository_account.issue_index = 0;
    repository_account.repository_token_pool_account =
        ctx.accounts.repository_token_pool_account.key();

    emit!(RepositoryCreated {
        repository_creator: repository_verified_user.user_pubkey.key(),
        repository_account: repository_account.key(),
        uri: repository_account.uri.clone(),
        rewards_mint: rewards_mint_account.key(),
        name: repository_account.name.clone(),
        description: repository_account.description.clone(),
        gh_usernames: gh_usernames.clone(),
        claim_amounts: claim_amounts.clone(),
    });

    Ok(())
}
