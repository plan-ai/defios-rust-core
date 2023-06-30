use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ReplaceLeafArg {
    pub index: u32,
    pub root: String,
    pub previous_leaf: String,
    pub new_leaf: String,
}
