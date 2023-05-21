use crate::error::DefiOSError;
use crate::helper::calculate_mint;
use crate::state::CommunalAccount;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create, get_associated_token_address, AssociatedToken, Create},
    token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(solana_amount:u64)]
pub struct BuyToken<'info> {
    #[account(mut,constraint = buyer.to_account_info().lamports() >= solana_amount @DefiOSError::InsufficientFunds)]
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
    pub communal_deposit: Account<'info, CommunalAccount>,
    #[account(mut)]
    pub communal_token_account: Account<'info, TokenAccount>,
    ///CHECK: Check for this account done in function call
    #[account(mut)]
    pub buyer_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub mint_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub rewards_mint: Account<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyToken>, solana_amount: u64) -> Result<()> {
    let token_program = &ctx.accounts.token_program;
    let buyer = &mut ctx.accounts.buyer;
    let communal_deposit = &mut ctx.accounts.communal_deposit;
    let communal_token_account = &mut ctx.accounts.communal_token_account;
    let buyer_token_account = &mut ctx.accounts.buyer_token_account;
    let rewards_mint = &mut ctx.accounts.rewards_mint;
    let system_program = &ctx.accounts.system_program;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let mint_authority = &mut ctx.accounts.mint_authority;

    let token_supply: u64;
    {
        let account_info = &rewards_mint.to_account_info();
        let data = &*account_info.try_borrow_data()?;
        let bytes_data = &mut &**data;
        token_supply = Mint::try_deserialize_unchecked(bytes_data).unwrap().supply;
    }
    let number_of_tokens = calculate_mint(token_supply, solana_amount);
    let rewards_key = rewards_mint.key();
    //execute function to send native sol amount to communal deposits
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &buyer.key(),
        &communal_deposit.key(),
        solana_amount,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[buyer.to_account_info(), communal_deposit.to_account_info()],
    )?;

    //checks if buyer has token account else creates it
    if buyer_token_account.data_is_empty() {
        create(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: buyer.to_account_info(),
                associated_token: buyer_token_account.to_account_info(),
                authority: buyer.to_account_info(),
                mint: rewards_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }

    let expected_buyer_token_account =
        get_associated_token_address(&buyer.key(), &rewards_mint.key());
    require!(
        expected_buyer_token_account.eq(&buyer_token_account.key()),
        DefiOSError::TokenAccountMismatch
    );

    //mints required number of tokens
    // Create the MintTo struct for our context
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"are_we_conscious",
        b"is love life ?  ",
        b"arewemadorinlove",
        rewards_key.as_ref(),
        &[communal_deposit.bump],
    ]];

    let cpi_accounts = MintTo {
        mint: rewards_mint.to_account_info(),
        to: communal_token_account.to_account_info(),
        authority: mint_authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    // Create the CpiContext we need for the request
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    // Execute anchor's helper function to mint tokens
    mint_to(cpi_ctx, number_of_tokens)?;
    //transfers token to buyer

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
