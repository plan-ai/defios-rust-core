use crate::constants::MAX_INT;
use crate::error::DefiOSError;
use crate::helper::verify_calc_sell;
use crate::state::{CommunalAccount, Repository,DefaultVestingSchedule};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token,
    token::{transfer, Burn, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(lamports_amount:u64,number_of_tokens:u64)]
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
    #[account(mut)]
    pub repository_account: Account<'info, Repository>,
    pub token_program: Program<'info, Token>,
    #[account(mut,seeds = [b"Miners",
    b"MinerC",
    repository_account.key().as_ref()],
    bump)]
    pub rewards_mint: Account<'info, Mint>,
    #[account(seeds = [
        b"isGodReal?",
        b"DoULoveMe?",
        b"SweetChick"
    ],
    bump=default_schedule.bump,
    )]
    pub default_schedule: Account<'info, DefaultVestingSchedule>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<SellToken>, lamports_amount: u64, number_of_tokens: u64) -> Result<()> {
    let rewards_mint = &ctx.accounts.rewards_mint;
    let token_program = &ctx.accounts.token_program;
    let communal_deposit = &mut ctx.accounts.communal_deposit;
    let communal_token_account = &mut ctx.accounts.communal_token_account;
    let seller = &mut ctx.accounts.seller;
    let seller_token_account = &mut ctx.accounts.seller_token_account;
    let default_schedule = &ctx.accounts.default_schedule;

    let total = (default_schedule.number_of_schedules as u64) * default_schedule.per_vesting_amount;
    //get supply of token
    let token_supply: u64;
    {
        let account_info = &rewards_mint.to_account_info();
        let data = &*account_info.try_borrow_data()?;
        let bytes_data = &mut &**data;
        token_supply = Mint::try_deserialize_unchecked(bytes_data).unwrap().supply;
    }

    require!(
        (number_of_tokens as u128) < MAX_INT,
        DefiOSError::MathOverflow
    );
    require!(
        verify_calc_sell(token_supply-total, lamports_amount, number_of_tokens),
        DefiOSError::IncorrectMaths
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

    //execute function to send native sol amount to seller
    let communal_info = &communal_deposit.to_account_info();
    **communal_info.try_borrow_mut_lamports()? -= lamports_amount;
    **seller.try_borrow_mut_lamports()? += lamports_amount;

    Ok(())
}
