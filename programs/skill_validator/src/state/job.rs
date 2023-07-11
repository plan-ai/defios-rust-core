use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
#[repr(u8)]
pub enum JobLength {
    CoupleOfDays,
    ShortTerm,
    MediumTerm,
    LongTerm,
}

#[account]
#[derive(InitSpace)]
pub struct Job {
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
    #[max_len(250)]
    pub appliers: Vec<Pubkey>,
}
