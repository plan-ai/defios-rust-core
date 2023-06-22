use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ReviewerType {
    JobCreator,
    Freelancer,
}

#[event]
pub struct JobReviewed {
    pub reviewer: Pubkey,
    pub reviewer_type: ReviewerType,
    pub review: String,
}
