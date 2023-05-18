use crate::helper::calculate_mint;
use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token::{transfer, Approve, Token, TokenAccount, Transfer,Mint},
    associated_token::AssociatedToken
};
use crate::state::{CommunalAccount};

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut,
        seeds = [
            b"are_we_conscious",
            b"is love life ?  ",
            b"arewemadorinlove",
            rewards_mint.key().as_ref()
        ],
    bump
    )]
    pub communal_deposit: Account<'info,CommunalAccount>,
    #[account(mut)]
    pub communal_token_account:Account<'info,TokenAccount>,
    #[account(
        mut,
        constraint = buyer_token_account.owner.eq(&buyer.key()),
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub rewards_mint: Account<'info,Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyToken>, solana_amount: u64) -> Result<()> {
    let token_program = &ctx.accounts.token_program;
    let buyer = &ctx.accounts.buyer;
    let communal_deposit = &ctx.accounts.communal_deposit;
    let communal_token_account = &mut ctx.accounts.communal_token_account;
    let buyer_token_account = &ctx.accounts.buyer_token_account;
    let rewards_mint = &mut ctx.accounts.rewards_mint;
    let system_program = &ctx.accounts.system_program;

    let account_info = &rewards_mint.to_account_info();
    let data = &*account_info.try_borrow_data()?;
    let bytes_data = &mut &**data;
    let token_supply = Mint::try_deserialize_unchecked(bytes_data).unwrap().supply;
    let number_of_tokens = calculate_mint(token_supply, solana_amount);
    //execute function to send native sol amount to communal deposits
    transfer(
        CpiContext::new(
            system_program.to_account_info(),
            Transfer {
                from: buyer.to_account_info(),
                to: communal_deposit.to_account_info(),
                authority: buyer.to_account_info()
            },
        ),
        solana_amount,
    )?;
    //transfers token to buyer
    let rewards_key = rewards_mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"are_we_conscious",
        b"is love life ?  ",
        b"arewemadorinlove",
        rewards_key.as_ref(),
        &[communal_deposit.bump],
    ]];
    
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: communal_token_account.to_account_info(),
                to: buyer_token_account.to_account_info(),
                authority: communal_deposit.to_account_info(),
            },
            signer_seeds,
        ),
        number_of_tokens,
    )?;
    Ok(())
}
