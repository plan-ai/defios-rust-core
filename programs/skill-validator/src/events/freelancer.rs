use anchor_lang::prelude::*;

#[event]
pub struct VerifiedFreelancerAdded {
    pub router_creator: Pubkey,
    pub name_router_account: Pubkey,
    pub verified_user_account: Pubkey,
    pub user_metadata_uri: String,
    pub user_pubkey: Pubkey,
}
