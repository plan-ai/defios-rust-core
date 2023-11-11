use anchor_lang::prelude::{constant, Pubkey};
use solana_program::pubkey;

#[constant]
pub const AUTHORIZED_PUBLIC_KEY: Pubkey = pubkey!("55kBY9yxqSC42boV8PywT2gqGzgLi5MPAtifNRgPNezF");
#[constant]
pub const MAX_INT: u128 = u128::pow(2, 64) - 1;
#[constant]
pub const VOTING_END: i64 = 72 * 60 * 60;
#[constant]
pub const TRUSTED_NAME_ROUTERS: &[Pubkey] = &[AUTHORIZED_PUBLIC_KEY];
