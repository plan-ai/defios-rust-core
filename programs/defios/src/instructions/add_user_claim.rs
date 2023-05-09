use crate::state::{NameRouter, Repository, UserClaim};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(user_name: String)]
pub struct AddUserClaim<'info> {
    #[account(
        mut,
        address = name_router_account.router_creator
    )]
    pub router_creator: Signer<'info>,

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
        init_if_needed,
        payer = router_creator,
        space = UserClaim::size(),
        seeds = [
            b"user_claim",
            user_name.as_bytes(),
            repository_account.key().as_ref(),
            name_router_account.key().as_ref(),
        ],
        bump,
    )]
    pub user_claim_account: Account<'info, UserClaim>,
    pub repository_account: Account<'info, Repository>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddUserClaim>, user_name: String, claim_amount: u64) -> Result<()> {
    let user_claim_account = &mut ctx.accounts.user_claim_account;
    let name_router_account = &ctx.accounts.name_router_account;
    let repository_account = &ctx.accounts.repository_account;

    user_claim_account.bump = *ctx.bumps.get("user_claim_account").unwrap();
    user_claim_account.token_amount += claim_amount;
    user_claim_account.gh_user = user_name;
    user_claim_account.is_claimed = false;
    user_claim_account.repository_account = repository_account.key();
    user_claim_account.name_router_account = name_router_account.key();

    Ok(())
}
