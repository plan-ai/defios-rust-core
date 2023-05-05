use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token::{Token,Approve},
};
use crate::helper::calculate_mint;

#[derive(Accounts)]
pub struct BuyToken<'info> {
    ///CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    ///CHECK: This is not dangerous because we don't read or write from this account
    pub delegate: AccountInfo<'info>,
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<BuyToken>) -> Result<()> {
    let amount = calculate_mint(1,1);

    let cpi_accounts = Approve {
        to: ctx.accounts.to.to_account_info(),
        delegate: ctx.accounts.delegate.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    // Create the CpiContext we need for the request
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Execute anchor's helper function to approve tokens
    token::approve(cpi_ctx, amount)?;
    Ok(())
} 