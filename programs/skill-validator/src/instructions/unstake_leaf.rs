use crate::error::ApplicationError;
use crate::helpers::Bytes;
use crate::{LeafStake, LeafUnStaked};
use anchor_lang::prelude::*;
use anchor_spl::{
    mint::USDC,
    token::{close_account, transfer, CloseAccount, Mint, Token, TokenAccount, Transfer},
};
#[derive(Accounts)]
#[instruction(unstake_amount:u64)]
pub struct UnStakeLeaf<'info> {
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
        constraint = authority_usdc_account.owner == authority.key()
    )]
    pub authority_usdc_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [
        b"Stak",    
        &stake_account.leaf.leading_bits().to_be_bytes(),
        merkle_tree.key().as_ref(),
        &stake_account.index.to_be_bytes()
        ],
        bump=stake_account.bump,
        constraint = stake_account.stake_amount>=unstake_amount
    )]
    pub stake_account: Account<'info, LeafStake>,
    #[account(address=USDC)]
    pub usdc_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = stake_account_usdc.mint.eq(&usdc_mint.key()),
        constraint = stake_account_usdc.owner.eq(&stake_account.key()),
        constraint = stake_account_usdc.amount >= unstake_amount@ApplicationError::CantUnstakeMoreThanStake
    )]
    pub stake_account_usdc: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<UnStakeLeaf>, unstake_amount: u64) -> Result<()> {
    let token_program = &ctx.accounts.token_program;
    let authority = &mut ctx.accounts.authority;
    let authority_usdc_account = &mut ctx.accounts.authority_usdc_account;
    let stake_account = &mut ctx.accounts.stake_account;
    let merkle_tree = &ctx.accounts.merkle_tree;
    let stake_account_usdc = &mut ctx.accounts.stake_account_usdc;

    let merkle_tree_key = merkle_tree.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"issue",
        b"Stak",
        &stake_account.leaf.leading_bits().to_be_bytes(),
        merkle_tree_key.as_ref(),
        &stake_account.index.to_be_bytes(),
        &[stake_account.bump],
    ]];

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: stake_account_usdc.to_account_info(),
                to: authority_usdc_account.to_account_info(),
                authority: stake_account.to_account_info(),
            },
            signer_seeds,
        ),
        unstake_amount,
    )?;

    stake_account.stake_amount -= unstake_amount;

    emit!(LeafUnStaked {
        leaf: stake_account.leaf,
        index: stake_account.index,
        unstake_amount: unstake_amount,
        tree: merkle_tree.key()
    });

    if stake_account.stake_amount == 0 {
        close_account(CpiContext::new_with_signer(
            token_program.to_account_info(),
            CloseAccount {
                account: stake_account_usdc.to_account_info(),
                authority: stake_account.to_account_info(),
                destination: authority.to_account_info(),
            },
            signer_seeds,
        ))?;

        close_account(CpiContext::new(
            token_program.to_account_info(),
            CloseAccount {
                account: stake_account.to_account_info(),
                authority: authority.to_account_info(),
                destination: authority.to_account_info(),
            },
        ))?;
    };

    Ok(())
}
