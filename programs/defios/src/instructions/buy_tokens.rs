use crate::constants::MAX_INT;
use crate::error::DefiOSError;
use crate::helper::verify_calc_buy;
use crate::state::{CommunalAccount, DefaultVestingSchedule, Repository};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create, get_associated_token_address, AssociatedToken, Create},
    mint::USDC,
    token,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(usdc_amount:u64)]
pub struct BuyToken<'info> {
    #[account(mut,constraint = buyer.to_account_info().lamports() >= usdc_amount @DefiOSError::InsufficientFunds)]
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
    #[account(
        mut,
        constraint=communal_token_account.mint==rewards_mint.key(),
        constraint = communal_token_account.owner == communal_deposit.key()
    )]
    pub communal_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint=communal_usdc_account.mint==usdc_mint.key(),
        constraint = communal_usdc_account.owner == communal_deposit.key()
    )]
    pub communal_usdc_account: Account<'info, TokenAccount>,
    ///CHECK: Check for this account done in function call
    #[account(mut)]
    pub buyer_token_account: AccountInfo<'info>,
    #[account(
        mut,
        constraint=buyer_usdc_account.mint==usdc_mint.key(),
        constraint = buyer_usdc_account.owner == buyer.key()
    )]
    pub buyer_usdc_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub repository_account: Box<Account<'info, Repository>>,
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
    #[account(
        seeds = [
            b"isGodReal?",
            b"DoULoveMe?",
            b"SweetChick"
        ],
        bump=default_schedule.bump,
    )]
    pub default_schedule: Box<Account<'info, DefaultVestingSchedule>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyToken>, usdc_amount: u64, number_of_tokens: u64) -> Result<()> {
    let token_program = &ctx.accounts.token_program;
    let buyer = &mut ctx.accounts.buyer;
    let communal_deposit = &mut ctx.accounts.communal_deposit;
    let communal_token_account = &mut ctx.accounts.communal_token_account;
    let buyer_token_account = &mut ctx.accounts.buyer_token_account;
    let rewards_mint = &mut ctx.accounts.rewards_mint;
    let system_program = &ctx.accounts.system_program;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let repository_account = &ctx.accounts.repository_account;
    let default_schedule = &ctx.accounts.default_schedule;
    let buyer_usdc_account = &mut ctx.accounts.buyer_usdc_account;
    let communal_usdc_account = &mut ctx.accounts.communal_usdc_account;

    let total = (default_schedule.number_of_schedules as u64) * default_schedule.per_vesting_amount;
    let token_supply = rewards_mint.supply;
    let modified_token_supply: u64 =
        (token_supply - total) / (u64::pow(10, rewards_mint.decimals.into()));
    let modified_tokens: u64 = number_of_tokens / (u64::pow(10, rewards_mint.decimals.into()));
    require!(
        (number_of_tokens as u128) < MAX_INT,
        DefiOSError::MathOverflow
    );
    require!(
        verify_calc_buy(modified_token_supply, usdc_amount, modified_tokens),
        DefiOSError::IncorrectMaths
    );
    let rewards_key = rewards_mint.key();
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
    //execute function to send usdc to communal deposits
    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: buyer_usdc_account.to_account_info(),
                to: communal_usdc_account.to_account_info(),
                authority: buyer.to_account_info(),
            },
        ),
        usdc_amount,
    )?;

    //mints required number of tokens
    let bump = *ctx.bumps.get("rewards_mint").unwrap();
    let repository_account_key = repository_account.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"Miners",
        b"MinerC",
        repository_account_key.as_ref(),
        &[bump],
    ]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: rewards_mint.to_account_info(),
                to: communal_token_account.to_account_info(),
                authority: rewards_mint.to_account_info(),
            },
            signer_seeds,
        ),
        number_of_tokens,
    )?;
    //transfers token to buyer
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
                from: communal_token_account.to_account_info(),
                to: buyer_token_account.to_account_info(),
                authority: communal_deposit.to_account_info(),
            },
            communal_signer_seeds,
        ),
        number_of_tokens,
    )?;

    Ok(())
}
