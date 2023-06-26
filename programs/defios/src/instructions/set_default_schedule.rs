use crate::constants::AUTHORIZED_PUBLIC_KEY;
use crate::error::DefiOSError;
use crate::state::{DefaultVestingSchedule, DefaultVestingScheduleChanged};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AdminDefaultVestingScheduleShift<'info> {
    ///CHECK: This is not dangerous public key constraint is already set
    #[account(mut, signer,constraint=AUTHORIZED_PUBLIC_KEY.eq(&authority.key()) @DefiOSError::UnauthorizedActionAttempted)]
    pub authority: AccountInfo<'info>,
    #[account(init_if_needed,
    payer = authority,
    seeds = [
        b"isGodReal?",
        b"DoULoveMe?",
        b"SweetChick"
    ],
    bump,
    space = 8+DefaultVestingSchedule::INIT_SPACE,
    )]
    pub default_schedule: Account<'info, DefaultVestingSchedule>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AdminDefaultVestingScheduleShift>,
    number_of_schedules: u32,
    per_vesting_amount: u64,
    unix_change: u64,
) -> Result<()> {
    let default_schedule = &mut ctx.accounts.default_schedule;
    default_schedule.bump = *ctx.bumps.get("default_schedule").unwrap();
    default_schedule.number_of_schedules = number_of_schedules;
    default_schedule.per_vesting_amount = per_vesting_amount;
    default_schedule.unix_change = unix_change;

    emit!(DefaultVestingScheduleChanged {
        number_of_schedules: number_of_schedules,
        per_vesting_amount: per_vesting_amount,
        unix_change: unix_change,
    });

    Ok(())
}
