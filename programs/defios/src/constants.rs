use anchor_lang::prelude::Pubkey;
use solana_program::pubkey;

pub const AUTHORIZED_PUBLIC_KEY: Pubkey = pubkey!("55kBY9yxqSC42boV8PywT2gqGzgLi5MPAtifNRgPNezF");
pub const CONSTANT_OF_PROPORTIONALITY: i128 = 100; //THIS VALUE IS MULTIPLIED BY 1000x to maintain support for 3 decimal places
