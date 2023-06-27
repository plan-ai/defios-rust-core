use crate::event::NameRouterCreated;
use crate::state::NameRouter;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(signing_domain: String, signature_version: u8)]
pub struct CreateNameRouter<'info> {
    #[account(mut)]
    pub router_creator: Signer<'info>,

    #[account(
        init,
        payer = router_creator,
        space = 8+NameRouter::INIT_SPACE,
        seeds = [
            signing_domain.as_bytes(),
            signature_version.to_string().as_bytes(),
            router_creator.key().as_ref()
        ],
        bump
    )]
    pub name_router_account: Account<'info, NameRouter>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateNameRouter>,
    signing_domain: String,
    signature_version: u8,
) -> Result<()> {
    let name_router_account = &mut ctx.accounts.name_router_account;
    let router_creator = &ctx.accounts.router_creator;

    name_router_account.bump = *ctx.bumps.get("name_router_account").unwrap();
    name_router_account.router_creator = router_creator.key();
    name_router_account.signing_domain = signing_domain;
    name_router_account.signature_version = signature_version;
    name_router_account.total_verified_users = 0;
    emit!(NameRouterCreated {
        router_creator: router_creator.key(),
        name_router_account: name_router_account.key(),
    });
    Ok(())
}
