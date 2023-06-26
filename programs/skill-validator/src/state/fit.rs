use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ValidatedFit {
    pub bump: u8,
    pub validator: Pubkey,
    #[max_len(1000)]
    pub freelancers: Vec<Pubkey>,
    pub job: Pubkey,
}
