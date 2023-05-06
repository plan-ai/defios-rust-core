use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token::{Token,Burn,Mint,Transfer,transfer,TokenAccount},
};
use crate::helper::calculate_burn;

#[derive(Accounts)]
pub struct SellToken<'info> {
    ///CHECK: Communal deposit account
    #[account(mut)]
    pub communal_account: AccountInfo<'info>,
    /// CHECK: This is the token that we want to mint
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    /// CHECK: This is the token account that we want to burn tokens from
    #[account(mut)]
    pub from: AccountInfo<'info>,
    #[account(
        mut,
        constraint = from_token_account.owner.eq(&from.key()),
    )]
    pub from_token_account: Account<'info, TokenAccount>,
    /// CHECK: the authority of the mint account
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<SellToken>,amount:u128) -> Result<()> {
    
    let mint = &ctx.accounts.mint;
    let from = &ctx.accounts.from;
    let authority = &ctx.accounts.authority;
    let token_program = &ctx.accounts.token_program;
    let communal_account = &ctx.accounts.communal_account;
    let from_token_account = &ctx.accounts.from_token_account;
    
    let transfer_amount = calculate_burn(1,amount);
    
    let cpi_accounts = Burn {
        mint: mint.to_account_info(),
        from: from_token_account.to_account_info(),
        authority: authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    // Create the CpiContext we need for the request
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Execute anchor's helper function to burn tokens
    token::burn(cpi_ctx, amount.try_into().unwrap())?;

    //execute function to send amount to users
    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: communal_account.to_account_info(),
                to: from_token_account.to_account_info(),
                authority: authority.to_account_info(),
            },
        ),
        transfer_amount,
    )?;

    Ok(())
}
