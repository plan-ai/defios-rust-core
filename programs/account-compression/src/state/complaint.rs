use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Complaint {
    pub bump: u8,
    pub job: Pubkey,
    #[max_len(1000)]
    pub complaint: String,
    pub accepted: bool,
}

#[event]
pub struct ComplaintCreated {
    pub job: Pubkey,
    pub complaint: String,
}

#[event]
pub struct ComplaintAccepted {
    pub complaint: Pubkey,
}
