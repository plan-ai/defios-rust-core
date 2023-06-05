use crate::error::DefiOSError;
use crate::helper::verify_swap;
use crate::state::{CommunalAccount, Repository};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create, get_associated_token_address, AssociatedToken, Create},
    token,
    token::{transfer, Burn, Mint, Token, TokenAccount, Transfer},
};

#[derive(Accounts)]
#[instruction(token_amount_1:u64)]
pub struct SwapToken<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut,
        seeds = [
            b"are_we_conscious",
            b"is love life ?  ",
            b"arewemadorinlove",
            rewards_mint1.key().as_ref()
        ],
    bump
    )]
    pub communal_deposit1: Account<'info, CommunalAccount>,
    #[account(mut,constraint = communal_token_account.mint==rewards_mint1.key())]
    pub communal_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut,constraint = buyer_token_account1.amount >= token_amount_1,constraint = buyer_token_account1.mint==rewards_mint1.key())]
    pub buyer_token_account1: Box<Account<'info, TokenAccount>>,
    ///CHECK: Check for this account done in function call
    #[account(mut)]
    pub buyer_token_account2: AccountInfo<'info>,
    #[account(mut)]
    pub repository_account1: Account<'info, Repository>,
    #[account(mut,seeds = [b"Miners",
    b"MinerC",
    repository_account1.key().as_ref()],
    bump)]
    pub rewards_mint1: Account<'info, Mint>,
    #[account(mut)]
    pub repository_account2: Account<'info, Repository>,
    #[account(mut,seeds = [b"Miners",
    b"MinerC",
    repository_account2.key().as_ref()],
    bump)]
    pub rewards_mint2: Box<Account<'info, Mint>>,
    #[account(mut,
        seeds = [
            b"are_we_conscious",
            b"is love life ?  ",
            b"arewemadorinlove",
            rewards_mint2.key().as_ref()
        ],
    bump
    )]
    pub communal_deposit2: Account<'info, CommunalAccount>,
    #[account(mut,constraint = communal_token_account2.mint==rewards_mint2.key())]
    pub communal_token_account2: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<SwapToken>, token_amount_1: u64, token_amount_2: u64) -> Result<()> {
    //gets all the token and system programs needed
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;
    let associated_token_program = &ctx.accounts.associated_token_program;
    //gets all the needed rewards mints
    let rewards_mint1 = &mut ctx.accounts.rewards_mint1;
    let rewards_mint2 = &mut ctx.accounts.rewards_mint2;
    //all the buyer accounts
    let buyer = &mut ctx.accounts.buyer;
    let buyer_token_account1 = &mut ctx.accounts.buyer_token_account1;
    let buyer_token_account2 = &mut ctx.accounts.buyer_token_account2;
    //check for buyer token account 2
    if buyer_token_account2.data_is_empty() {
        create(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: buyer.to_account_info(),
                associated_token: buyer_token_account2.to_account_info(),
                authority: buyer.to_account_info(),
                mint: rewards_mint2.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?
    };
    //gets the communal deposits necesccay
    let communal_deposit1 = &mut ctx.accounts.communal_deposit1;
    let communal_deposit2 = &mut ctx.accounts.communal_deposit2;
    let communal_token_account1 = &mut ctx.accounts.communal_token_account;
    let communal_token_account2 = &mut ctx.accounts.communal_token_account2;
    //gets needed repository accounts
    let repository_account2 = &ctx.accounts.repository_account2;
    //gets supply of both the tokend
    let token_supply1: u64;
    {
        let account_info = &rewards_mint1.to_account_info();
        let data = &*account_info.try_borrow_data()?;
        let bytes_data = &mut &**data;
        token_supply1 = Mint::try_deserialize_unchecked(bytes_data).unwrap().supply;
    }
    let token_supply2: u64;
    {
        let account_info = &rewards_mint2.to_account_info();
        let data = &*account_info.try_borrow_data()?;
        let bytes_data = &mut &**data;
        token_supply2 = Mint::try_deserialize_unchecked(bytes_data).unwrap().supply;
    }
    //checks correct communal token accounts were sent
    let expected_buyer_token_account2 =
        get_associated_token_address(&buyer.key(), &rewards_mint2.key());
    require!(
        expected_buyer_token_account2.eq(&buyer_token_account2.key()),
        DefiOSError::TokenAccountMismatch
    );
    //checks values were correctly calculated
    require!(
        verify_swap(token_supply1, token_supply2, token_amount_1, token_amount_2),
        DefiOSError::IncorrectMaths
    );
    //calculated seeds of communal deposts
    let bump2 = *ctx.bumps.get("rewards_mint2").unwrap();
    let repository_account_key2 = repository_account2.key();
    let rewards_key1 = rewards_mint1.key();
    let rewards_key2 = rewards_mint2.key();
    let signer_seeds2: &[&[&[u8]]] = &[&[
        b"Miners",
        b"MinerC",
        repository_account_key2.as_ref(),
        &[bump2],
    ]];
    let communal_signer_seeds: &[&[&[u8]]] = &[&[
        b"are_we_conscious",
        b"is love life ?  ",
        b"arewemadorinlove",
        rewards_key1.as_ref(),
        &[communal_deposit1.bump],
    ]];
    let communal_signer_seeds2: &[&[&[u8]]] = &[&[
        b"are_we_conscious",
        b"is love life ?  ",
        b"arewemadorinlove",
        rewards_key2.as_ref(),
        &[communal_deposit2.bump],
    ]];
    //transfers token 1 to communal deposit
    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: buyer_token_account1.to_account_info(),
                to: communal_token_account1.to_account_info(),
                authority: buyer.to_account_info(),
            },
        ),
        token_amount_1,
    )?;
    //burns received token 1 away
    let cpi_accounts = Burn {
        mint: rewards_mint1.to_account_info(),
        from: communal_token_account1.to_account_info(),
        authority: communal_deposit1.to_account_info(),
    };
    let cpi_program = token_program.to_account_info();
    // Create the CpiContext we need for the request
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, communal_signer_seeds);

    //Execute anchor's helper function to burn tokens
    token::burn(cpi_ctx, token_amount_1)?;
    //mints required number of token amount 2
    token::mint_to(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            token::MintTo {
                mint: rewards_mint2.to_account_info(),
                to: communal_token_account2.to_account_info(),
                authority: rewards_mint2.to_account_info(),
            },
            signer_seeds2,
        ),
        token_amount_2,
    )?;
    //sends required number of token 2 to user
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: communal_token_account2.to_account_info(),
                to: buyer_token_account2.to_account_info(),
                authority: communal_deposit2.to_account_info(),
            },
            communal_signer_seeds2,
        ),
        token_amount_2,
    )?;
    Ok(())
}
