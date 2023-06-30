use crate::error::{AccountCompressionError, ApplicationError};
use crate::{
    fill_in_proof_from_canopy, merkle_tree_get_size, zero_copy::ZeroCopy, ChangeLogEvent,
    ConcurrentMerkleTreeHeader, LeafStake, LeafStaked, CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create as create_associated_token_account, AssociatedToken, Create},
    mint::USDC,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use spl_concurrent_merkle_tree::concurrent_merkle_tree::ConcurrentMerkleTree;
#[derive(Accounts)]
#[instruction(index:u32,leaf:[u8;32],root:[u8;32],stake_amount:u64)]
pub struct StakeLeaf<'info> {
    #[account(mut)]
    /// CHECK: This account is validated in the instruction
    pub merkle_tree: UncheckedAccount<'info>,

    /// Authority that controls write-access to the tree
    /// Typically a program, e.g., the Bubblegum contract validates that leaves are valid NFTs.
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        constraint=authority_usdc_account.mint==usdc_mint.key(),
        constraint = authority_usdc_account.owner == authority.key(),
        constraint = authority_usdc_account.amount >= stake_amount @ApplicationError::InsufficientBalance
    )]
    pub authority_usdc_account: Account<'info, TokenAccount>,
    #[account(init_if_needed,
        payer = authority,
        space = 8 + LeafStake::INIT_SPACE,
        seeds = [
        b"Stak",    
        merkle_tree.key().as_ref(),
        &index.to_be_bytes()
        ],
        bump
    )]
    pub stake_account: Account<'info, LeafStake>,
    ///CHECK: Handling of account is done in function
    #[account(mut)]
    pub stake_account_usdc: UncheckedAccount<'info>,
    #[account(address=USDC)]
    pub usdc_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<StakeLeaf>,
    index: u32,
    leaf: [u8; 32],
    root: [u8; 32],
    stake_amount: u64,
) -> Result<()> {
    let token_program = &ctx.accounts.token_program;
    let authority_usdc_account = &mut ctx.accounts.authority_usdc_account;
    let authority = &mut ctx.accounts.authority;
    let stake_account = &mut ctx.accounts.stake_account;
    let merkle_tree = &ctx.accounts.merkle_tree;
    let stake_account_usdc = &mut ctx.accounts.stake_account_usdc;
    let usdc_mint = &ctx.accounts.usdc_mint;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let system_program = &ctx.accounts.system_program;

    require_eq!(
        *merkle_tree.owner,
        crate::id(),
        AccountCompressionError::IncorrectAccountOwner
    );
    let merkle_tree_bytes = merkle_tree.try_borrow_data()?;
    let (header_bytes, rest) = merkle_tree_bytes.split_at(CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1);

    let header = ConcurrentMerkleTreeHeader::try_from_slice(header_bytes)?;
    header.assert_valid()?;
    header.assert_valid_leaf_index(index)?;

    let merkle_tree_size = merkle_tree_get_size(&header)?;
    let (tree_bytes, canopy_bytes) = rest.split_at(merkle_tree_size);

    let mut proof = vec![];
    for node in ctx.remaining_accounts.iter() {
        proof.push(node.key().to_bytes());
    }
    fill_in_proof_from_canopy(canopy_bytes, header.get_max_depth(), index, &mut proof)?;
    let id = ctx.accounts.merkle_tree.key();

    merkle_tree_apply_fn!(header, id, tree_bytes, prove_leaf, root, leaf, &proof, index)?;

    //Creating token account if empty
    if stake_account_usdc.data_is_empty() {
        create_associated_token_account(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: authority.to_account_info(),
                associated_token: stake_account_usdc.to_account_info(),
                authority: stake_account.to_account_info(),
                mint: usdc_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }

    stake_account.bump = *ctx.bumps.get("stake_account").unwrap();
    stake_account.root = root;
    stake_account.leaf = leaf;
    stake_account.index = index;
    stake_account.stake_amount += stake_amount;
    stake_account.tree = merkle_tree.key();

    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: authority_usdc_account.to_account_info(),
                to: stake_account_usdc.to_account_info(),
                authority: authority.to_account_info(),
            },
        ),
        stake_amount,
    )?;

    emit!(LeafStaked {
        leaf: leaf,
        index: index,
        stake_amount: stake_amount,
        tree: merkle_tree.key()
    });

    Ok(())
}
