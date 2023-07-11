use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct NameRouter {
    pub bump: u8,
    pub signature_version: u8,
    pub total_verified_users: u64,
    pub router_creator: Pubkey,
    #[max_len(50)]
    pub signing_domain: String,
}
