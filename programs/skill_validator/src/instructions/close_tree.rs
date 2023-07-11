use crate::error::AccountCompressionError;
use crate::{
    merkle_tree_get_size, zero_copy::ZeroCopy, ChangeLogEvent, ConcurrentMerkleTreeHeader,
    CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1,
};
use anchor_lang::prelude::*;
use spl_concurrent_merkle_tree::concurrent_merkle_tree::ConcurrentMerkleTree;
/// Context for closing a tree
#[derive(Accounts)]
pub struct CloseTree<'info> {
    #[account(mut)]
    /// CHECK: This account is validated in the instruction
    pub merkle_tree: AccountInfo<'info>,

    /// Authority that controls write-access to the tree
    pub authority: Signer<'info>,

    /// CHECK: Recipient of funds after
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
}

pub fn handler(ctx: Context<CloseTree>) -> Result<()> {
    require_eq!(
        *ctx.accounts.merkle_tree.owner,
        crate::id(),
        AccountCompressionError::IncorrectAccountOwner
    );
    let mut merkle_tree_bytes = ctx.accounts.merkle_tree.try_borrow_mut_data()?;
    let (header_bytes, rest) =
        merkle_tree_bytes.split_at_mut(CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1);

    let header = ConcurrentMerkleTreeHeader::try_from_slice(header_bytes)?;
    header.assert_valid_authority(&ctx.accounts.authority.key())?;

    let merkle_tree_size = merkle_tree_get_size(&header)?;
    let (tree_bytes, canopy_bytes) = rest.split_at_mut(merkle_tree_size);

    let id = ctx.accounts.merkle_tree.key();
    merkle_tree_apply_fn_mut!(header, id, tree_bytes, prove_tree_is_empty,)?;

    // Close merkle tree account
    // 1. Move lamports
    let dest_starting_lamports = ctx.accounts.recipient.lamports();
    **ctx.accounts.recipient.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(ctx.accounts.merkle_tree.lamports())
        .unwrap();
    **ctx.accounts.merkle_tree.lamports.borrow_mut() = 0;

    // 2. Set all CMT account bytes to 0
    header_bytes.fill(0);
    tree_bytes.fill(0);
    canopy_bytes.fill(0);

    Ok(())
}
