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
pub mod constants;
pub mod error;
pub mod events;
#[macro_use]
pub mod macros;
pub mod helpers;
mod noop;
pub mod state;
pub mod zero_copy;

use crate::canopy::{fill_in_proof_from_canopy, update_canopy};
pub use crate::error::AccountCompressionError;
pub use crate::events::{
    AccountCompressionEvent, ApplicationDataEvent, ApplicationDataEventV1, ChangeLogEvent,
    LeafStaked, LeafUnStaked, ReviewerType,
};
use crate::noop::wrap_event;
pub use crate::noop::{wrap_application_data_v1, Noop};
use crate::state::{
    merkle_tree_get_size, ConcurrentMerkleTreeHeader, JobLength, LeafStake,
    CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1,
};

pub mod instructions;
use instructions::*;

/// Exported for Anchor / Solita
pub use spl_concurrent_merkle_tree::{
    concurrent_merkle_tree::ConcurrentMerkleTree, error::ConcurrentMerkleTreeError, node::Node,
};

declare_id!("2HDrBtAm848yy8Swv92bzQdS8fmBjTbwJoN9vHLvvim7");

#[program]
pub mod skill_validator {
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

    pub fn append_leaf(ctx: Context<AppendLeaf>, leaf: [u8; 32], data: String) -> Result<()> {
        append_leaf::handler(ctx, leaf, data)
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
        data: String,
    ) -> Result<()> {
        insert_append_leaf::handler(ctx, root, leaf, index, data)
    }

    pub fn add_job(
        ctx: Context<AddJob>,
        job_name: String,
        job_desc: String,
        job_length: JobLength,
        job_metadata_uri: String,
    ) -> Result<()> {
        add_job::handler(ctx, job_name, job_desc, job_length, job_metadata_uri)
    }

    pub fn stake_job(ctx: Context<StakeJob>, stake_amount: u64) -> Result<()> {
        stake_job::handler(ctx, stake_amount)
    }

    pub fn close_job(ctx: Context<CloseJob>) -> Result<()> {
        close_job::handler(ctx)
    }

    pub fn add_review(
        ctx: Context<AddReview>,
        reviewer_type: ReviewerType,
        review: String,
        review_no: u16,
    ) -> Result<()> {
        add_review::handler(ctx, reviewer_type, review, review_no)
    }

    pub fn accept_job(ctx: Context<AcceptJob>) -> Result<()> {
        accept_job::handler(ctx)
    }

    pub fn raise_complaint(ctx: Context<RaiseComplaint>, complaint_text: String) -> Result<()> {
        raise_complaint::handler(ctx, complaint_text)
    }

    pub fn accept_complaint(ctx: Context<AcceptComplaint>) -> Result<()> {
        accept_complaint::handler(ctx)
    }

    pub fn stake_leaf(
        ctx: Context<StakeLeaf>,
        leaf: [u8; 32],
        root: [u8; 32],
        index: u32,
        stake_amount: u64,
    ) -> Result<()> {
        stake_leaf::handler(ctx, leaf, root, index, stake_amount)
    }

    pub fn unstake_leaf(ctx: Context<UnStakeLeaf>, unstake_amount: u64) -> Result<()> {
        unstake_leaf::handler(ctx, unstake_amount)
    }

    pub fn add_name_router(
        ctx: Context<CreateNameRouter>,
        signing_domain: String,
        signature_version: u8,
    ) -> Result<()> {
        add_name_router::handler(ctx, signing_domain, signature_version)
    }

    pub fn add_verified_freelancer(
        ctx: Context<AddVerifiedFreelancer>,
        user_metadata_uri: String,
        user_pubkey: Pubkey,
        msg: Vec<u8>,
        sig: [u8; 64],
    ) -> Result<()> {
        add_verified_freelancer::handler(ctx, user_metadata_uri, user_pubkey, msg, sig)
    }

    pub fn apply_job(ctx: Context<ApplyJob>) -> Result<()> {
        apply_job::handler(ctx)
    }

    pub fn create_skill(
        ctx: Context<CreateSkill>,
        roots: Box<Vec<Vec<u8>>>,
        leafs: Box<Vec<Vec<u8>>>,
        indexes: Box<Vec<u32>>,
        merkle_trees: Box<Vec<Pubkey>>,
    ) -> Result<()> {
        create_skill::handler(ctx, roots, leafs, indexes, merkle_trees)
    }

    pub fn destroy_skill(ctx: Context<DestroySkill>) -> Result<()> {
        destroy_skill::handler(ctx)
    }

    pub fn validate_fit(ctx: Context<ValidateFit>) -> Result<()> {
        validate_fit::handler(ctx)
    }

    pub fn index_data(ctx: Context<IndexData>, metadata_uris: Vec<String>) -> Result<()> {
        index_data::handler(ctx, metadata_uris)
    }

    pub fn accept_freelancer(ctx: Context<AcceptFreelancer>) -> Result<()> {
        accept_freelancer::handler(ctx)
    }

    pub fn claim_freelance_reward(ctx: Context<ClaimFreelanceReward>) -> Result<()> {
        claim_freelance_reward::handler(ctx)
    }
}
