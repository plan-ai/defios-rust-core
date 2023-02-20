use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create, AssociatedToken, Create},
    token::{Mint, Token, TokenAccount},
};

use crate::state::VestingSchedule;

#[derive(Accounts)]
#[instruction(number_of_schedules: u64)]
pub struct Register<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = VestingSchedule::size(number_of_schedules),
        seeds = [
            b"vesting",
            token_mint.key().as_ref(),
            authority.key().as_ref()
        ],
        bump,
    )]
    pub vesting_account: Account<'info, VestingSchedule>,

    pub token_mint: Account<'info, Mint>,

    #[account(
        constraint = destination_token_account.mint.eq(&token_mint.key())
    )]
    pub destination_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA check is done at the handler function
    #[account(mut)]
    pub vesting_token_account: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<Register>, number_of_schedules: u64) -> Result<()> {
    let vesting_account = &mut ctx.accounts.vesting_account;
    let authority = &ctx.accounts.authority;
    let rent = &ctx.accounts.rent;

    // Token accounts
    let token_mint = &ctx.accounts.token_mint;
    let destination_token_account = &ctx.accounts.destination_token_account;
    let vesting_token_account = &ctx.accounts.vesting_token_account;

    // Program accounts
    let token_program = &ctx.accounts.token_program;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let system_program = &ctx.accounts.system_program;

    if vesting_token_account.data_is_empty() {
        create(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: authority.to_account_info(),
                associated_token: vesting_token_account.to_account_info(),
                authority: vesting_account.to_account_info(),
                mint: token_mint.to_account_info(),
                rent: rent.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }

    vesting_account.bump = *ctx.bumps.get("vesting_account").unwrap();
    vesting_account.is_initialized = false;
    vesting_account.authority = authority.key();
    vesting_account.destination_address = destination_token_account.key();
    vesting_account.mint_address = token_mint.key();
    vesting_account.max_schedules = number_of_schedules;
    vesting_account.schedules = vec![];

    Ok(())
}
