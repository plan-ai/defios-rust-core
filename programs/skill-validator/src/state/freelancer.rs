use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Freelancer {
    pub bump: u8,
    pub name_router: Pubkey,
    #[max_len(100)]
    pub user_metadata_uri: String,
    pub user_pubkey: Pubkey,
}
