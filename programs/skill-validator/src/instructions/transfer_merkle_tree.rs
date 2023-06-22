use crate::error::AccountCompressionError;
use crate::{ConcurrentMerkleTreeHeader, CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1};
use anchor_lang::prelude::*;
/// Context for transferring `authority`
#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    #[account(mut)]
    /// CHECK: This account is validated in the instruction
    pub merkle_tree: UncheckedAccount<'info>,

    /// Authority that controls write-access to the tree
    /// Typically a program, e.g., the Bubblegum contract validates that leaves are valid NFTs.
    pub authority: Signer<'info>,
}

/// Transfers `authority`.
/// Requires `authority` to sign
pub fn handler(ctx: Context<TransferAuthority>, new_authority: Pubkey) -> Result<()> {
    require_eq!(
        *ctx.accounts.merkle_tree.owner,
        crate::id(),
        AccountCompressionError::IncorrectAccountOwner
    );
    let mut merkle_tree_bytes = ctx.accounts.merkle_tree.try_borrow_mut_data()?;
    let (mut header_bytes, _) =
        merkle_tree_bytes.split_at_mut(CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1);

    let mut header = ConcurrentMerkleTreeHeader::try_from_slice(header_bytes)?;
    header.assert_valid_authority(&ctx.accounts.authority.key())?;

    header.set_new_authority(&new_authority);
    header.serialize(&mut header_bytes)?;

    Ok(())
}
