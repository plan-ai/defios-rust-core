use crate::error::{AccountCompressionError,ApplicationError};
use crate::{
    merkle_tree_get_size, update_canopy, wrap_event, zero_copy::ZeroCopy, AccountCompressionEvent,
    ChangeLogEvent, ConcurrentMerkleTreeHeader, Noop, CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1
};
use crate::state::freelancer::Freelancer;
use anchor_lang::prelude::*;
use spl_concurrent_merkle_tree::concurrent_merkle_tree::ConcurrentMerkleTree;
/// Context for initializing a new SPL ConcurrentMerkleTree
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(zero)]
    /// CHECK: This account will be zeroed out, and the size will be validated
    pub merkle_tree: UncheckedAccount<'info>,

    /// Authority that controls write-access to the tree
    /// Typically a program, e.g., the Bubblegum contract validates that leaves are valid NFTs.
    pub authority: Signer<'info>,

    /// Program used to emit changelogs as cpi instruction data.
    pub noop: Program<'info, Noop>,
    ///CHECK: Check done in constraint level
    pub freelancer: AccountInfo<'info>,
    #[account(
        constraint = verified_freelancer_account.user_pubkey == freelancer.key()@ApplicationError::UnauthorizedJobAction)]
    pub verified_freelancer_account: Account<'info, Freelancer>,
}

/// Creates a new merkle tree with maximum leaf capacity of `power(2, max_depth)`
/// and a minimum concurrency limit of `max_buffer_size`.
///
/// Concurrency limit represents the # of replace instructions that can be successfully
/// executed with proofs dated for the same root. For example, a maximum buffer size of 1024
/// means that a minimum of 1024 replaces can be executed before a new proof must be
/// generated for the next replace instruction.
///
/// Concurrency limit should be determined by empirically testing the demand for
/// state built on top of SPL Compression.
///
/// For instructions on enabling the canopy, see [canopy].
/// Note:
/// Supporting this instruction open a security vulnerability for indexers.
/// This instruction has been deemed unusable for publicly indexed compressed NFTs.
/// Indexing batched data in this way requires indexers to read in the `uri`s onto physical storage
/// and then into their database. This opens up a DOS attack vector, whereby this instruction is
/// repeatedly invoked, causing indexers to fail.
///
/// Because this instruction was deemed insecure, this instruction has been removed
/// until secure usage is available on-chain.
// pub fn init_merkle_tree_with_root(
//     ctx: Context<Initialize>,
//     max_depth: u32,
//     max_buffer_size: u32,
//     root: [u8; 32],
//     leaf: [u8; 32],
//     index: u32,
//     _changelog_db_uri: String,
//     _metadata_db_uri: String,
// ) -> Result<()> {
//     require_eq!(
//         *ctx.accounts.merkle_tree.owner,
//         crate::id(),
//         AccountCompressionError::IncorrectAccountOwner
//     );
//     let mut merkle_tree_bytes = ctx.accounts.merkle_tree.try_borrow_mut_data()?;

//     let (mut header_bytes, rest) =
//         merkle_tree_bytes.split_at_mut(CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1);

//     let mut header = ConcurrentMerkleTreeHeader::try_from_slice(&header_bytes)?;
//     header.initialize(
//         max_depth,
//         max_buffer_size,
//         &ctx.accounts.authority.key(),
//         Clock::get()?.slot,
//     );
//     header.serialize(&mut header_bytes)?;
//     let merkle_tree_size = merkle_tree_get_size(&header)?;
//     let (tree_bytes, canopy_bytes) = rest.split_at_mut(merkle_tree_size);

//     // Get rightmost proof from accounts
//     let mut proof = vec![];
//     for node in ctx.remaining_accounts.iter() {
//         proof.push(node.key().to_bytes());
//     }
//     fill_in_proof_from_canopy(canopy_bytes, header.max_depth, index, &mut proof)?;
//     assert_eq!(proof.len(), max_depth as usize);

//     let id = ctx.accounts.merkle_tree.key();
//     // A call is made to ConcurrentMerkleTree::initialize_with_root(root, leaf, proof, index)
//     let change_log = merkle_tree_apply_fn!(
//         header,
//         id,
//         tree_bytes,
//         initialize_with_root,
//         root,
//         leaf,
//         &proof,
//         index
//     )?;
//     wrap_event(change_log.try_to_vec()?, &ctx.accounts.log_wrapper)?;
//     update_canopy(canopy_bytes, header.max_depth, Some(change_log))
// }

pub fn handler(ctx: Context<Initialize>, max_depth: u32, max_buffer_size: u32) -> Result<()> {
    require_eq!(
        *ctx.accounts.merkle_tree.owner,
        crate::id(),
        AccountCompressionError::IncorrectAccountOwner
    );
    let mut merkle_tree_bytes = ctx.accounts.merkle_tree.try_borrow_mut_data()?;

    let (mut header_bytes, rest) =
        merkle_tree_bytes.split_at_mut(CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1);

    let mut header = ConcurrentMerkleTreeHeader::try_from_slice(header_bytes)?;
    header.initialize(
        max_depth,
        max_buffer_size,
        &ctx.accounts.authority.key(),
        Clock::get()?.slot,
    );
    header.serialize(&mut header_bytes)?;
    let merkle_tree_size = merkle_tree_get_size(&header)?;
    let (tree_bytes, canopy_bytes) = rest.split_at_mut(merkle_tree_size);
    let id = ctx.accounts.merkle_tree.key();
    let change_log_event = merkle_tree_apply_fn_mut!(header, id, tree_bytes, initialize,ctx.accounts.verified_freelancer_account.user_pubkey)?;
    wrap_event(
        &AccountCompressionEvent::ChangeLog(*change_log_event),
        &ctx.accounts.noop,
    )?;
    update_canopy(canopy_bytes, header.get_max_depth(), None)
}
