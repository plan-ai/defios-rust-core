use crate::{
    error::DefiOSError,
    state::{NameRouter, VerifiedUser},
};
use anchor_lang::prelude::*;
use anchor_spl::token::Token;

#[derive(Accounts)]
pub struct VerifyUserMetadata<'info> {
    #[account(
        mut,
        address = metadata_verifier_user.user_pubkey @ DefiOSError::UnauthorizedUser,
    )]
    pub metadata_verifier: Signer<'info>,
    #[account(
        seeds = [
            metadata_verifier_user.user_name.as_bytes(),
            metadata_verifier.key().as_ref(),
            name_router_account.key().as_ref()
        ],
        bump = metadata_verifier_user.bump
    )]
    pub metadata_verifier_user: Account<'info, VerifiedUser>,
    #[account(
        mut,
        address = verified_user.user_pubkey @ DefiOSError::UnauthorizedUser,
    )]
    ///CHECK: Verified user checks done in subsquent lines
    pub user: AccountInfo<'info>,
    #[account(
        seeds = [
            verified_user.user_name.as_bytes(),
            user.key().as_ref(),
            name_router_account.key().as_ref()
        ],
        bump = verified_user.bump
    )]
    pub verified_user: Account<'info, VerifiedUser>,
    #[account(
        address = metadata_verifier_user.name_router @ DefiOSError::InvalidNameRouter,
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
    ///CHECK: Handling of account is done in function
    #[account(mut)]
    pub metadata_token_pool_account: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<VerifyUserMetadata>) -> Result<()> {
    Ok(())
}
