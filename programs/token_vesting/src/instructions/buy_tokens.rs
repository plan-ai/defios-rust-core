use crate::helper::calculate_mint;
use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token::{transfer, Approve, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
pub struct BuyToken<'info> {
    ///CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub to: Signer<'info>,
    ///CHECK: Communal deposit account
    #[account(mut,
        seeds = [
            b"are_we_conscious",
            b"is love life ?  ",
            b"arewemadorinlove"
        ],
    bump
    )]
    pub communal_account: AccountInfo<'info>,
    #[account(
        mut,
        constraint = to_token_account.owner.eq(&to.key()),
    )]
    pub to_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    ///CHECK: This is not dangerous because we don't read or write from this account
    pub delegate: AccountInfo<'info>,
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<BuyToken>, amount: u128) -> Result<()> {
    let buy_amount = calculate_mint(1, amount);
    let token_program = &ctx.accounts.token_program;
    let to = &ctx.accounts.to;
    let communal_account = &ctx.accounts.communal_account;
    let delegate = &ctx.accounts.delegate;
    let authority = &ctx.accounts.authority;
    let to_token_account = &ctx.accounts.to_token_account;
    //execute function to send amount to communal deposits
    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: to_token_account.to_account_info(),
                to: communal_account.to_account_info(),
                authority: to.to_account_info(),
            },
        ),
        buy_amount,
    )?;

    let cpi_accounts = Approve {
        to: to.to_account_info(),
        delegate: delegate.to_account_info(),
        authority: authority.to_account_info(),
    };
    let cpi_program = token_program.to_account_info();
    // Create the CpiContext we need for the request
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Execute anchor's helper function to approve tokens
    token::approve(cpi_ctx, amount.try_into().unwrap())?;
    Ok(())
}
