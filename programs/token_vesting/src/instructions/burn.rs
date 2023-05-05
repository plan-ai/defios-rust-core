use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token::{Token,Burn,Mint},
};
use crate::helper::calculate_burn;

#[derive(Accounts)]
pub struct BurnToken<'info> {
    /// CHECK: This is the token that we want to mint
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    /// CHECK: This is the token account that we want to burn tokens from
    #[account(mut)]
    pub from: AccountInfo<'info>,
    /// CHECK: the authority of the mint account
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<BurnToken>) -> Result<()> {
    
    let amount = calculate_burn(1,1);

    let cpi_accounts = Burn {
        mint: ctx.accounts.mint.to_account_info(),
        from: ctx.accounts.from.to_account_info(),
        authority: ctx.accounts.authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    // Create the CpiContext we need for the request
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Execute anchor's helper function to burn tokens
    token::burn(cpi_ctx, amount)?;
    Ok(())
}
