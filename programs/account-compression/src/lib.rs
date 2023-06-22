//! SPL Account Compression is an on-chain program that exposes an interface to manipulating SPL ConcurrentMerkleTrees
//!
//! A buffer of proof-like changelogs is stored on-chain that allow multiple proof-based writes to succeed within the same slot.
//! This is accomplished by fast-forwarding out-of-date (or possibly invalid) proofs based on information stored in the changelogs.
//! See a copy of the whitepaper [here](https://drive.google.com/file/d/1BOpa5OFmara50fTvL0VIVYjtg-qzHCVc/view)
//!
//! To circumvent proof size restrictions stemming from Solana transaction size restrictions,
//! SPL Account Compression also provides the ability to cache the upper most leaves of the
//! concurrent merkle tree. This is called the "canopy", and is stored at the end of the
//! ConcurrentMerkleTreeAccount. More information can be found in the initialization instruction
//! documentation.
//!
//! While SPL ConcurrentMerkleTrees can generically store arbitrary information,
//! one exemplified use-case is the [Bubblegum](https://github.com/metaplex-foundation/metaplex-program-library/tree/master/bubblegum) contract,
//! which uses SPL-Compression to store encoded information about NFTs.
//! The use of SPL-Compression within Bubblegum allows for:
//! - up to 1 billion NFTs to be stored in a single account on-chain (>10,000x decrease in on-chain cost)
//! - up to 2048 concurrent updates per slot
//!
//! Operationally, SPL ConcurrentMerkleTrees **must** be supplemented by off-chain indexers to cache information
//! about leafs and to power an API that can supply up-to-date proofs to allow updates to the tree.
//! All modifications to SPL ConcurrentMerkleTrees are settled on the Solana ledger via instructions against the SPL Compression contract.
//! A production-ready indexer (Plerkle) can be found in the [Metaplex program library](https://github.com/metaplex-foundation/digital-asset-validator-plugin)

use anchor_lang::{prelude::*, solana_program::sysvar::rent::Rent};
use borsh::BorshDeserialize;

pub mod canopy;
pub mod error;
pub mod events;
#[macro_use]
pub mod macros;
mod noop;
pub mod state;
pub mod zero_copy;

pub use crate::noop::{wrap_application_data_v1, Noop};

use crate::canopy::{fill_in_proof_from_canopy, update_canopy};
pub use crate::error::AccountCompressionError;
pub use crate::events::{AccountCompressionEvent, ChangeLogEvent};
use crate::noop::wrap_event;
use crate::state::{
    merkle_tree_get_size, ConcurrentMerkleTreeHeader, CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1,
};

pub mod instructions;
use instructions::*;

/// Exported for Anchor / Solita
pub use spl_concurrent_merkle_tree::{
    concurrent_merkle_tree::ConcurrentMerkleTree, error::ConcurrentMerkleTreeError, node::Node,
};

declare_id!("cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK");

#[program]
pub mod spl_account_compression {
    use super::*;

    pub fn init_empty_merkle_tree(
        ctx: Context<Initialize>,
        max_depth: u32,
        max_buffer_size: u32,
    ) -> Result<()> {
        initialize_merkle_tree::handler(ctx, max_depth, max_buffer_size)
    }

    pub fn close_tree(ctx: Context<CloseTree>) -> Result<()> {
        close_tree::handler(ctx)
    }

    pub fn replace_leaf(
        ctx: Context<ReplaceLeaf>,
        root: [u8; 32],
        previous_leaf: [u8; 32],
        new_leaf: [u8; 32],
        index: u32,
    ) -> Result<()> {
        replace_leaf::handler(ctx, root, previous_leaf, new_leaf, index)
    }

    pub fn append_leaf(ctx: Context<AppendLeaf>, leaf: [u8; 32]) -> Result<()> {
        append_leaf::handler(ctx, leaf)
    }

    pub fn transfer_tree(ctx: Context<TransferAuthority>, new_authority: Pubkey) -> Result<()> {
        transfer_merkle_tree::handler(ctx, new_authority)
    }

    pub fn verify_leaf(
        ctx: Context<VerifyLeaf>,
        root: [u8; 32],
        leaf: [u8; 32],
        index: u32,
    ) -> Result<()> {
        verify_leaf::handler(ctx, root, leaf, index)
    }

    pub fn insert_or_append_leaf(
        ctx: Context<Modify>,
        root: [u8; 32],
        leaf: [u8; 32],
        index: u32,
    ) -> Result<()> {
        insert_append_leaf::handler(ctx, root, leaf, index)
    }
}
