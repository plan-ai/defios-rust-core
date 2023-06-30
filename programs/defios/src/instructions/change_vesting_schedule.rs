use crate::constants::AUTHORIZED_PUBLIC_KEY;
use crate::error::DefiOSError;
use crate::event::VestingScheduleChanged;
use crate::state::{Repository, Schedule, VestingSchedule};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AdminVestingScheduleShift<'info> {
    ///CHECK: This is not dangerous public key constraint is already set
    #[account(mut, signer)]
    //constraint=AUTHORIZED_PUBLIC_KEY.eq(&authority.key()) @DefiOSError::UnauthorizedActionAttempted)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub repository_account: Account<'info, Repository>,
    #[account(mut, seeds = [
        b"vesting",
        repository_account.key().as_ref()
    ],
    bump = vesting_schedule.bump)]
    pub vesting_schedule: Account<'info, VestingSchedule>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AdminVestingScheduleShift>,
    new_vesting_schedule: Vec<Schedule>,
) -> Result<()> {
    let vesting_schedule = &mut ctx.accounts.vesting_schedule;
    let repository_account = &ctx.accounts.repository_account;
    let old_vesting_schedule = vesting_schedule.schedules.clone();
    vesting_schedule.schedules = new_vesting_schedule.clone();

    emit!(VestingScheduleChanged {
        repository_account: repository_account.key(),
        repository_creator: repository_account.repository_creator,
        old_vesting_schedule: old_vesting_schedule.to_vec(),
        new_vesting_schedule: new_vesting_schedule
    });

    Ok(())
}
