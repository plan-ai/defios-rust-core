use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Leaf {
    pub index: u32,
    pub root: String,
    pub leaf: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GraphData {
    pub from: Leaf,
    pub new: String,
    pub merkle_tree: Pubkey,
}
