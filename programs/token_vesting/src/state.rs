use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct VestingSchedule {
    pub bump: u8,
    pub is_initialized: bool,
    pub max_schedules: u64,
    pub authority: Pubkey,
    pub destination_address: Pubkey,
    pub mint_address: Pubkey,
    pub schedules: Vec<Schedule>,
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Schedule {
    pub release_time: u64,
    pub amount: u64,
}

impl VestingSchedule {
    pub fn size(number_of_schedules: u64) -> usize {
        let number_of_schedules = if number_of_schedules > 0 {
            number_of_schedules
        } else {
            1
        };

        8 + // discriminator
        1 + // bump
        1 + // is_initialized
        8 + // max_schedules
        32 + // authority
        32 + // destination_address
        32 + // mint_address
        number_of_schedules as usize * Schedule::size()
    }
}

impl Schedule {
    pub fn size() -> usize {
        4 + // Vec length discriminator
        8 + // release_time
        8 // amount
    }
}
