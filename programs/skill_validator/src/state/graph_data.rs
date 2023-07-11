use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Leaf {
    pub index: u32,
    pub leaf: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GraphData {
    pub from: Leaf,
    pub to: Leaf,
    pub root: [u8; 32],
}
