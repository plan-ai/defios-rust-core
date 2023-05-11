use anchor_lang::prelude::*;
pub mod error;
pub mod helper;
pub mod instructions;
pub mod state;

use crate::instructions::*;
use crate::state::*;

declare_id!("5Wm1LaCnGUY5S1om7qUy7PC6FnKRBCwxPLNeA97w17Hs");

#[program]
pub mod token_vesting {

    use super::*;

    pub fn register(ctx: Context<Register>, number_of_schedules: u64) -> Result<()> {
        register::handler(ctx, number_of_schedules)
    }

    pub fn add_schedules(ctx: Context<AddSchedules>, schedules: Vec<Schedule>) -> Result<()> {
        add_schedules::handler(ctx, schedules)
    }

    pub fn unlock_tokens(ctx: Context<UnlockTokens>) -> Result<()> {
        unlock_tokens::handler(ctx)
    }

    pub fn change_destination(ctx: Context<ChangeDestination>) -> Result<()> {
        change_destination::handler(ctx)
    }

    pub fn sell_tokens(ctx: Context<SellToken>, amount: u128) -> Result<()> {
        sell_tokens::handler(ctx, amount)
    }

    pub fn buy_tokens(ctx: Context<BuyToken>, amount: u128) -> Result<()> {
        buy_tokens::handler(ctx, amount)
    }

    pub fn create_communal_account(ctx: Context<RegisterCommunalAccount>) -> Result<()> {
        create_communal_account::handler(ctx)
    }
}
