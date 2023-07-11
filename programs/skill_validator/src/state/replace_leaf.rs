use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ReplaceLeafArg {
    pub index: u32,
    pub root: [u8; 32],
    pub previous_leaf: [u8; 32],
    pub new_leaf: String,
}
