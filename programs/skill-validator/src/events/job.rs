use crate::state::JobLength;
use anchor_lang::prelude::*;

#[event]
pub struct JobCreated {
    pub job_length: JobLength,
    pub job_name: String,
    pub job_desc: String,
    pub job_metadata_uri: String,
    pub job_creator: Pubkey,
}

#[event]
pub struct JobStaked {
    pub job: Pubkey,
    pub stake_amount: u64,
    pub unix_time: i64,
}

#[event]
pub struct JobClosed {
    pub job: Pubkey,
}

#[event]
pub struct JobAccepted {
    pub job: Pubkey,
}
