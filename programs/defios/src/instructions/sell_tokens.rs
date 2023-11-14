use crate::constants::{MAX_INT, TOKEN_VEST_AMOUNT, VESTING_NUMBER};
use crate::error::DefiOSError;
use crate::helper::verify_calc_sell;
use crate::state::{CommunalAccount, Repository};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create, get_associated_token_address, AssociatedToken, Create},
    mint::USDC,
    token,
    token::{transfer, Burn, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(usdc_amount:u64,number_of_tokens:u64)]
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
    #[account(
        mut,
        constraint = communal_token_account.mint==rewards_mint.key(),
        constraint = communal_token_account.owner == communal_deposit.key()
    )]
    pub communal_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = communal_usdc_account.mint==usdc_mint.key(),
        constraint = communal_usdc_account.owner == communal_deposit.key()
    )]
    pub communal_usdc_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = seller_token_account.amount >= number_of_tokens@DefiOSError::InsufficientFunds,
        constraint = seller_token_account.owner == seller.key(),
        constraint = seller_token_account.mint == rewards_mint.key()
    )]
    pub seller_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub repository_account: Box<Account<'info, Repository>>,
    ///CHECK: usdc account is setup in function
    #[account(mut)]
    pub seller_usdc_account: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    #[account(
        mut,
        seeds = [
            b"Miners",
            b"MinerC",
            repository_account.key().as_ref()
        ],
        bump
    )]
    pub rewards_mint: Account<'info, Mint>,
    // #[account(address=USDC)]
    pub usdc_mint: Account<'info, Mint>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SellToken>, usdc_amount: u64, number_of_tokens: u64) -> Result<()> {
    let rewards_mint = &ctx.accounts.rewards_mint;
    let token_program = &ctx.accounts.token_program;
    let communal_deposit = &mut ctx.accounts.communal_deposit;
    let communal_token_account = &mut ctx.accounts.communal_token_account;
    let seller = &mut ctx.accounts.seller;
    let seller_token_account = &mut ctx.accounts.seller_token_account;
    let seller_usdc_account = &mut ctx.accounts.seller_usdc_account;
    let usdc_mint = &ctx.accounts.usdc_mint;
    let communal_usdc_account = &mut ctx.accounts.communal_usdc_account;
    let system_program = &ctx.accounts.system_program;
    let associated_token_program = &ctx.accounts.associated_token_program;

    let total = VESTING_NUMBER * TOKEN_VEST_AMOUNT * u64::pow(10, rewards_mint.decimals.into());
    let token_supply = rewards_mint.supply;
    let modified_token_supply: u64 =
        (token_supply - total) / (u64::pow(10, rewards_mint.decimals.into()));
    let modified_tokens: u64 = number_of_tokens / (u64::pow(10, rewards_mint.decimals.into()));

    require!(
        (number_of_tokens as u128) < MAX_INT,
        DefiOSError::MathOverflow
    );
    require!(
        verify_calc_sell(modified_token_supply, usdc_amount, modified_tokens),
        DefiOSError::IncorrectMaths
    );
    //checks is seller usdc account exists, else creates it
    if seller_usdc_account.data_is_empty() {
        create(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: seller.to_account_info(),
                associated_token: seller_usdc_account.to_account_info(),
                authority: seller.to_account_info(),
                mint: usdc_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }
    let expected_seller_usdc_account =
        get_associated_token_address(&seller.key(), &usdc_mint.key());
    require!(
        expected_seller_usdc_account.eq(&seller_usdc_account.key()),
        DefiOSError::TokenAccountMismatch
    );

    //transfers spl token to communal token account
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

    //calculates signer seeds
    let rewards_key = rewards_mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"are_we_conscious",
        b"is love life ?  ",
        b"arewemadorinlove",
        rewards_key.as_ref(),
        &[communal_deposit.bump],
    ]];

    //burns incoming spl tokens

    let cpi_accounts = Burn {
        mint: rewards_mint.to_account_info(),
        from: communal_token_account.to_account_info(),
        authority: communal_deposit.to_account_info(),
    };
    let cpi_program = token_program.to_account_info();
    // Create the CpiContext we need for the request
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    //Execute anchor's helper function to burn tokens
    token::burn(cpi_ctx, number_of_tokens)?;

    //execute function to send usdc to seller
    let rewards_key = rewards_mint.key();
    let communal_signer_seeds: &[&[&[u8]]] = &[&[
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
                from: communal_usdc_account.to_account_info(),
                to: seller_usdc_account.to_account_info(),
                authority: communal_deposit.to_account_info(),
            },
            communal_signer_seeds,
        ),
        usdc_amount,
    )?;

    Ok(())
}
