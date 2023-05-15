use crate::constants::AUTHORIZED_PUBLIC_KEY;
use crate::error::DefiOSError;
use crate::state::{Repository, Schedule, VestingSchedule, VestingScheduleChanged};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as get_associated_token_address, AssociatedToken},
    token::Token,
};
#[derive(Accounts)]
pub struct AdminVestingScheduleShift<'info> {
    #[account(constraint=AUTHORIZED_PUBLIC_KEY.eq(&authority.key()),signer)]
    pub authority: AccountInfo<'info>,
    #[account(constraint=repository_account.vesting_schedule.eq(&vesting_schedule.key()))]
    pub repository_account: Account<'info, Repository>,
    #[account(
        constraint=repository_account.vesting_schedule.eq(&vesting_schedule.key())
    )]
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
