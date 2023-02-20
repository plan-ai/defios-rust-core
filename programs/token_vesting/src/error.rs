use anchor_lang::prelude::*;

#[error_code]
pub enum TokenVestingError {
    // 6000
    #[msg("Math overflow")]
    MathOverflow,

    // 6001
    #[msg("Token Mint mismatch")]
    MintMismatch,

    // 6002
    #[msg("Vesting account already initialized with schedules")]
    VestingAccountAlreadyInitialized,

    // 6003
    #[msg("Vesting account cannot have a close authority")]
    VestingAccountInvalidClose,

    // 6004
    #[msg("Vesting account cannot have a delegate")]
    VestingAccountInvalidDelegate,

    // 6005
    #[msg("Insufficient tokens to transfer")]
    InsufficientFunds,

    // 6006
    #[msg("Schedules addition limit reached")]
    SchedulesLimitReached,

    // 6007
    #[msg("Vesting contract has not reached release time")]
    VestingNotReachedRelease,
}
