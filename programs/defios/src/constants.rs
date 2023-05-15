use anchor_lang::prelude::Pubkey;
use solana_program::pubkey;

pub const NUMBER_OF_SCHEDULES: u64 = 4;
pub const VESTING_AMMOUNT: u64 = 10000;
pub const PER_VEST_AMOUNT: u64 = VESTING_AMMOUNT / NUMBER_OF_SCHEDULES;
pub const UNIX_CHANGE: u64 = (8 * 30 * 24 * 60 * 60) / NUMBER_OF_SCHEDULES;
pub const AUTHORIZED_PUBLIC_KEY: Pubkey = pubkey!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");