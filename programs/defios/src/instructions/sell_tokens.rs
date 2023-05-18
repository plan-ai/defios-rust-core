use crate::error::DefiOSError;
use crate::helper::calculate_burn;
use crate::state::CommunalAccount;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token,
    token::{transfer, Burn, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(number_of_tokens:u64)]
pub struct SellToken<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(mut,
        seeds = [
            b"are_we_conscious",
            b"is love life ?  ",
            b"arewemadorinlove",
            rewards_mint.key().as_ref()
        ],
    bump
    )]
    pub communal_deposit: Account<'info, CommunalAccount>,
    #[account(mut)]
    pub communal_token_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = seller_token_account.amount >= number_of_tokens@DefiOSError::InsufficientFunds)]
    pub seller_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub rewards_mint: Account<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SellToken>, number_of_tokens: u64) -> Result<()> {
    let rewards_mint = &ctx.accounts.rewards_mint;
    let token_program = &ctx.accounts.token_program;
    let communal_deposit = &mut ctx.accounts.communal_deposit;
    let communal_token_account = &mut ctx.accounts.communal_token_account;
    let seller = &mut ctx.accounts.seller;
    let seller_token_account = &mut ctx.accounts.seller_token_account;

    //get supply of token
    let token_supply:u64;
    {
        let account_info = &rewards_mint.to_account_info();
        let data = &*account_info.try_borrow_data()?;
        let bytes_data = &mut &**data;
        token_supply = Mint::try_deserialize_unchecked(bytes_data).unwrap().supply;
    }
    //get amount of solana to transfer
    let solana_amount = calculate_burn(token_supply, number_of_tokens);

    //transfers sol to communal token account
    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: seller_token_account.to_account_info(),
                to: communal_token_account.to_account_info(),
                authority: seller.to_account_info(),
            },
        ),
        number_of_tokens,
    )?;

    //execute function to send native sol amount to communal deposits
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &communal_deposit.key(),
        &seller.key(),
        solana_amount,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[communal_deposit.to_account_info(), seller.to_account_info()],
    )?;

    let cpi_accounts = Burn {
        mint: rewards_mint.to_account_info(),
        from: communal_token_account.to_account_info(),
        authority: rewards_mint.to_account_info(),
    };
    let cpi_program = token_program.to_account_info();
    // Create the CpiContext we need for the request
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    //Execute anchor's helper function to burn tokens
    token::burn(cpi_ctx, number_of_tokens)?;

    Ok(())
}
