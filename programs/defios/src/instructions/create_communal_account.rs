use crate::constants::AUTHORIZED_PUBLIC_KEY;
use crate::error::DefiOSError;
use crate::state::CommunalAccount;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create, get_associated_token_address, AssociatedToken, Create},
    mint::USDC,
    token::{Mint, Token},
};

#[derive(Accounts)]
pub struct RegisterCommunalAccount<'info> {
    ///CHECK: Authority can only have specified public key
    #[account(mut, signer)]
    //constraint=AUTHORIZED_PUBLIC_KEY.eq(&authority.key())@DefiOSError::UnauthorizedActionAttempted)]
    pub authority: AccountInfo<'info>,
    #[account(init_if_needed,
        payer = authority,
        space = CommunalAccount::size(),
        seeds = [
            b"are_we_conscious",
            b"is love life ?  ",
            b"arewemadorinlove",
            rewards_mint.key().as_ref()
        ],
    bump
    )]
    pub communal_deposit: Account<'info, CommunalAccount>,
    ///CHECK: This is handled in function body
    #[account(mut)]
    pub communal_token_account: UncheckedAccount<'info>,
    ///CHECK: This is handled in function body
    #[account(mut)]
    pub communal_usdc_account: UncheckedAccount<'info>,
    pub rewards_mint: Account<'info, Mint>,
    // #[account(address=USDC)]
    pub usdc_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterCommunalAccount>) -> Result<()> {
    let authority = &mut ctx.accounts.authority;
    let system_program = &ctx.accounts.system_program;
    let communal_deposit = &mut ctx.accounts.communal_deposit;
    let communal_token_account = &mut ctx.accounts.communal_token_account;
    let token_program = &ctx.accounts.token_program;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let rewards_mint = &ctx.accounts.rewards_mint;
    let communal_usdc_account = &ctx.accounts.communal_usdc_account;
    let usdc_mint = &ctx.accounts.usdc_mint;
    communal_deposit.bump = *ctx.bumps.get("communal_deposit").unwrap();
    //creates communal token account for new spl token
    if communal_token_account.data_is_empty() {
        create(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: authority.to_account_info(),
                associated_token: communal_token_account.to_account_info(),
                authority: communal_deposit.to_account_info(),
                mint: rewards_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }

    let expected_communal_token_account =
        get_associated_token_address(&communal_deposit.key(), &rewards_mint.key());
    require!(
        expected_communal_token_account.eq(&communal_token_account.key()),
        DefiOSError::TokenAccountMismatch
    );

    //creates communal token account for usdc
    if communal_usdc_account.data_is_empty() {
        create(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: authority.to_account_info(),
                associated_token: communal_usdc_account.to_account_info(),
                authority: communal_deposit.to_account_info(),
                mint: usdc_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }

    let expected_communal_usdc_account =
        get_associated_token_address(&communal_deposit.key(), &usdc_mint.key());
    require!(
        expected_communal_usdc_account.eq(&communal_usdc_account.key()),
        DefiOSError::TokenAccountMismatch
    );

    Ok(())
}
