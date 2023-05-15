use anchor_lang::prelude::*;
use crate::constants::AUTHORIZED_PUBLIC_KEY;

#[derive(Accounts)]
pub struct RegisterCommunalAccount<'info> {
    #[account(constrant=AUTHORIZED_PUBLIC_KEY.eq(&authority.pubkey()),signer)]
    pub authority: AccountInfo<'info>,
    ///CHECK: Communal deposit account
    #[account(init_if_needed,
        payer = authority,
        space = 9,
        seeds = [
            b"are_we_conscious",
            b"is love life ?  ",
            b"arewemadorinlove"
        ],
    bump
    )]
    pub communal_deposit: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterCommunalAccount>) -> Result<()> {
    Ok(())
}
