use crate::error::AccountCompressionError;
use crate::{
    fill_in_proof_from_canopy, merkle_tree_get_size, update_canopy, wrap_event,
    zero_copy::ZeroCopy, AccountCompressionEvent, ChangeLogEvent, ConcurrentMerkleTreeHeader, Noop,
    CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1,
};
use anchor_lang::prelude::*;
use spl_concurrent_merkle_tree::concurrent_merkle_tree::ConcurrentMerkleTree;
/// Context for inserting, appending, or replacing a leaf in the tree
///
/// Modification instructions also require the proof to the leaf to be provided
/// as 32-byte nodes via "remaining accounts".
#[derive(Accounts)]
pub struct Modify<'info> {
    #[account(mut)]
    /// CHECK: This account is validated in the instruction
    pub merkle_tree: UncheckedAccount<'info>,

    /// Authority that controls write-access to the tree
    /// Typically a program, e.g., the Bubblegum contract validates that leaves are valid NFTs.
    pub authority: Signer<'info>,

    /// Program used to emit changelogs as cpi instruction data.
    pub noop: Program<'info, Noop>,
}

/// This instruction takes a proof, and will attempt to write the given leaf
/// to the specified index in the tree. If the insert operation fails, the leaf will be `append`-ed
/// to the tree.
/// It is up to the indexer to parse the final location of the leaf from the emitted changelog.
pub fn handler(ctx: Context<Modify>, root: [u8; 32], leaf: [u8; 32], index: u32) -> Result<()> {
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
    header.assert_valid_leaf_index(index)?;

    let merkle_tree_size = merkle_tree_get_size(&header)?;
    let (tree_bytes, canopy_bytes) = rest.split_at_mut(merkle_tree_size);

    let mut proof = vec![];
    for node in ctx.remaining_accounts.iter() {
        proof.push(node.key().to_bytes());
    }
    fill_in_proof_from_canopy(canopy_bytes, header.get_max_depth(), index, &mut proof)?;
    // A call is made to ConcurrentMerkleTree::fill_empty_or_append
    let id = ctx.accounts.merkle_tree.key();
    let change_log_event = merkle_tree_apply_fn_mut!(
        header,
        id,
        tree_bytes,
        fill_empty_or_append,
        root,
        leaf,
        &proof,
        index,
    )?;
    update_canopy(
        canopy_bytes,
        header.get_max_depth(),
        Some(&change_log_event),
    )?;
    wrap_event(
        &AccountCompressionEvent::ChangeLog(*change_log_event),
        &ctx.accounts.noop,
    )
}
