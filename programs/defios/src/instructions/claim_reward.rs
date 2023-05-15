use crate::{error::DefiOSError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{
        create as create_associated_token_account, get_associated_token_address, AssociatedToken,
        Create,
    },
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use sha256::digest;

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(
        mut,
        constraint = commit_creator.key().eq(&commit_verified_user.user_pubkey) @ DefiOSError::UnauthorizedUser,
    )]
    pub commit_creator: Signer<'info>,

    /// CHECK: PDA check is done at the handler function
    #[account(mut)]
    pub commit_creator_reward_token_account: UncheckedAccount<'info>,
    #[account(address = repository_account.rewards_mint)]
    pub rewards_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        address = name_router_account.router_creator @ DefiOSError::UnauthorizedUser,
    )]
    pub router_creator: SystemAccount<'info>,

    #[account(
        seeds = [
            name_router_account.signing_domain.as_bytes(),
            name_router_account.signature_version.to_string().as_bytes(),
            router_creator.key().as_ref()
        ],
        bump = name_router_account.bump
    )]
    pub name_router_account: Box<Account<'info, NameRouter>>,

    #[account(
        seeds = [
            commit_verified_user.user_name.as_bytes(),
            commit_creator.key().as_ref(),
            name_router_account.key().as_ref(),
        ],
        bump = commit_verified_user.bump,
    )]
    pub commit_verified_user: Box<Account<'info, VerifiedUser>>,

    #[account(
        address = repository_account.repository_creator
    )]
    pub repository_creator: SystemAccount<'info>,

    #[account(
        address = issue_account.issue_creator
    )]
    pub issue_creator: SystemAccount<'info>,

    #[account(
        seeds = [
            b"repository",
            repository_account.name.as_bytes(),
            repository_creator.key().as_ref(),
        ],
        bump = repository_account.bump
    )]
    pub repository_account: Box<Account<'info, Repository>>,

    #[account(
        mut,
        seeds = [
            b"issue",
            issue_account.index.to_string().as_bytes(),
            repository_account.key().as_ref(),
            issue_creator.key().as_ref(),
        ],
        bump = issue_account.bump
    )]
    pub issue_account: Box<Account<'info, Issue>>,

    #[account(mut, address = issue_account.issue_token_pool_account)]
    pub issue_token_pool_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            b"commit",
            first_commit_account.commit_hash.as_bytes(),
            commit_creator.key().as_ref(),
            issue_account.key().as_ref(),
        ],
        bump = first_commit_account.bump
    )]
    pub first_commit_account: Box<Account<'info, Commit>>,

    #[account(
        seeds = [
            b"commit",
            second_commit_account.commit_hash.as_bytes(),
            commit_creator.key().as_ref(),
            issue_account.key().as_ref(),
        ],
        bump = second_commit_account.bump
    )]
    pub second_commit_account: Box<Account<'info, Commit>>,

    #[account(
        seeds = [
            b"commit",
            third_commit_account.commit_hash.as_bytes(),
            commit_creator.key().as_ref(),
            issue_account.key().as_ref(),
        ],
        bump = third_commit_account.bump
    )]
    pub third_commit_account: Box<Account<'info, Commit>>,

    #[account(
        seeds = [
            b"commit",
            fourth_commit_account.commit_hash.as_bytes(),
            commit_creator.key().as_ref(),
            issue_account.key().as_ref(),
        ],
        bump = fourth_commit_account.bump
    )]
    pub fourth_commit_account: Box<Account<'info, Commit>>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

pub fn hash(content: &String, creator_pubkey_str: Option<String>) -> Vec<u8> {
    let mut final_content = format!("{}{}", content, "");
    match creator_pubkey_str {
        Some(x) => {
            final_content = format!("{}{}", content, x);
        }
        None => {}
    };

    digest(final_content).as_bytes().to_vec()
}

pub fn handler(ctx: Context<ClaimReward>) -> Result<()> {
    let commit_creator = &mut ctx.accounts.commit_creator;
    let rewards_mint = &ctx.accounts.rewards_mint;
    let repository_account = &ctx.accounts.repository_account;
    let issue_account = &mut ctx.accounts.issue_account;
    let commit_creator_reward_token_account = &mut ctx.accounts.commit_creator_reward_token_account;
    let issue_token_pool_account = &mut ctx.accounts.issue_token_pool_account;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let system_program = &ctx.accounts.system_program;
    let token_program = &ctx.accounts.token_program;
    // Commit accounts
    let first_commit_account = &ctx.accounts.first_commit_account;
    let second_commit_account = &ctx.accounts.second_commit_account;
    let third_commit_account = &ctx.accounts.third_commit_account;
    let fourth_commit_account = &ctx.accounts.fourth_commit_account;

    //checking if issue token account sent is same as expected
    let expected_issue_token_pool_account =
        get_associated_token_address(&issue_account.key(), &rewards_mint.key());

    require!(
        expected_issue_token_pool_account.eq(&issue_token_pool_account.key()),
        DefiOSError::TokenAccountMismatch
    );

    //Creating token account if empty
    if commit_creator_reward_token_account.data_is_empty() {
        msg!("Creating Commit creator reward token account");
        create_associated_token_account(CpiContext::new(
            associated_token_program.to_account_info(),
            Create {
                payer: commit_creator.to_account_info(),
                associated_token: commit_creator_reward_token_account.to_account_info(),
                authority: commit_creator.to_account_info(),
                mint: rewards_mint.to_account_info(),
                system_program: system_program.to_account_info(),
                token_program: token_program.to_account_info(),
            },
        ))?;
    }

    // Checking tree hashes
    let first_sha_hash = hash(
        &first_commit_account.tree_hash,
        Some(commit_creator.key().to_string()),
    );

    let second_sha_hash = hash(&second_commit_account.tree_hash, None);

    let third_sha_hash = hash(&third_commit_account.tree_hash, None);

    let fourth_sha_hash = hash(
        &fourth_commit_account.tree_hash,
        Some(commit_creator.key().to_string()),
    );

    // require!(
    //      first_sha_hash.eq(&second_sha_hash) && third_sha_hash.eq(&fourth_sha_hash),
    //      DefiOSError::HashesMismatch,
    // );

    // Transferring pool balance to commit creator
    let issue_index_str = repository_account.issue_index.to_string();
    let repository_account_key = repository_account.key();
    let issue_creator_key = issue_account.issue_creator.key();

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"issue",
        issue_index_str.as_bytes(),
        repository_account_key.as_ref(),
        issue_creator_key.as_ref(),
        &[issue_account.bump],
    ]];

    let token_balance = issue_token_pool_account.amount;

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: issue_token_pool_account.to_account_info(),
                to: commit_creator_reward_token_account.to_account_info(),
                authority: issue_account.to_account_info(),
            },
            signer_seeds,
        ),
        token_balance,
    )?;

    Ok(())
}
