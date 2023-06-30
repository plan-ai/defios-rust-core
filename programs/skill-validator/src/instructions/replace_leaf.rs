use crate::error::AccountCompressionError;
use crate::state::replace_leaf::ReplaceLeafArg;
use crate::{
    fill_in_proof_from_canopy, merkle_tree_get_size, update_canopy, wrap_event,
    zero_copy::ZeroCopy, AccountCompressionEvent, ChangeLogEvent, ConcurrentMerkleTreeHeader, Noop,
    CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1,ApplicationDataEvent, ApplicationDataEventV1
};
use anchor_lang::prelude::*;
use solana_program::keccak::hashv;
use spl_concurrent_merkle_tree::concurrent_merkle_tree::ConcurrentMerkleTree;
/// Context for inserting, appending, or replacing a leaf in the tree
///
/// Modification instructions also require the proof to the leaf to be provided
/// as 32-byte nodes via "remaining accounts".
#[derive(Accounts)]
pub struct ReplaceLeaf<'info> {
    #[account(mut)]
    /// CHECK: This account is validated in the instruction
    pub merkle_tree: UncheckedAccount<'info>,

    /// Authority that controls write-access to the tree
    /// Typically a program, e.g., the Bubblegum contract validates that leaves are valid NFTs.
    pub authority: Signer<'info>,

    /// Program used to emit changelogs as cpi instruction data.
    pub noop: Program<'info, Noop>,
}

/// Executes an instruction that overwrites a leaf node.
/// Composing programs should check that the data hashed into previous_leaf
/// matches the authority information necessary to execute this instruction.
pub fn handler(ctx: Context<ReplaceLeaf>, replace_leaf: ReplaceLeafArg) -> Result<()> {
    require_eq!(
        *ctx.accounts.merkle_tree.owner,
        crate::id(),
        AccountCompressionError::IncorrectAccountOwner
    );
    let mut merkle_tree_bytes = ctx.accounts.merkle_tree.try_borrow_mut_data()?;
    let (header_bytes, rest) =
        merkle_tree_bytes.split_at_mut(CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1);

    let index = replace_leaf.index;
    let root = hashv(&[replace_leaf.root.as_bytes()]).to_bytes();
    let previous_leaf = hashv(&[replace_leaf.previous_leaf.as_bytes()]).to_bytes();
    let new_leaf = hashv(&[replace_leaf.new_leaf.as_bytes()]).to_bytes();
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
    let id = ctx.accounts.merkle_tree.key();
    // A call is made to ConcurrentMerkleTree::set_leaf(root, previous_leaf, new_leaf, proof, index)
    let change_log_event = merkle_tree_apply_fn_mut!(
        header,
        id,
        tree_bytes,
        set_leaf,
        root,
        previous_leaf,
        new_leaf,
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
    )?;
    wrap_event(
        &AccountCompressionEvent::ApplicationData(ApplicationDataEvent::V1(
            ApplicationDataEventV1 {
                application_data: replace_leaf.new_leaf.into_bytes(),
            },
        )),
        &ctx.accounts.noop,
    )
}
