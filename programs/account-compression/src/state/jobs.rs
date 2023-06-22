use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum JobLength {
    CoupleOfDays,
    ShortTerm,
    MediumTerm,
    LongTerm,
}

impl Space for JobLength {
    const INIT_SPACE: usize = 1;
}

#[account]
#[derive(InitSpace)]
pub struct Jobs {
    pub bump: u8,
    pub job_length: JobLength,
    #[max_len(50)]
    pub job_name: String,
    #[max_len(500)]
    pub job_desc: String,
    #[max_len(50)]
    pub job_metadata_uri: String,
    pub job_creator: Pubkey,
    pub job_stake: u64,
    pub assigned_freelancer: Option<Pubkey>,
    pub job_completed: bool,
}

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
