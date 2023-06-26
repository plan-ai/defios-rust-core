use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult, instruction::Instruction,
    pubkey::Pubkey,
};

declare_id!("B1TFYfUGbwg8vj74FxaWS3xhtQye7AFk7EnFysTuC6tV");

#[cfg(not(feature = "no-entrypoint"))]
use solana_program::entrypoint;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(noop);

pub fn noop(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    Ok(())
}

pub fn instruction(data: Vec<u8>) -> Instruction {
    Instruction {
        program_id: crate::id(),
        accounts: vec![],
        data,
    }
}
