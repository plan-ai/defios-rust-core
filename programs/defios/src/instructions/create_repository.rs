use crate::{
    error::DefiOSError,
    state::{
        DefaultVestingSchedule, NameRouter, Repository, RepositoryCreated, Schedule, VerifiedUser,
        VestingSchedule,
    },
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create, get_associated_token_address, AssociatedToken, Create},
    metadata::Metadata,
    token,
    token::{Mint, Token},
};
use mpl_token_metadata;
use mpl_token_metadata::{instruction as token_instruction, pda::find_metadata_account};
use solana_program::program::invoke_signed;

#[derive(Accounts)]
#[instruction(id: String)]
pub struct CreateRepository<'info> {
    #[account(
        mut,
        address = repository_verified_user.user_pubkey @ DefiOSError::UnauthorizedUser,
    )]
    pub repository_creator: Signer<'info>,

    #[account(
        seeds = [
            repository_verified_user.user_name.as_bytes(),
            repository_creator.key().as_ref(),
            name_router_account.key().as_ref()
        ],
        bump = repository_verified_user.bump
    )]
    pub repository_verified_user: Account<'info, VerifiedUser>,

    #[account(
        address = repository_verified_user.name_router @ DefiOSError::InvalidNameRouter,
        seeds = [
            name_router_account.signing_domain.as_bytes(),
            name_router_account.signature_version.to_string().as_bytes(),
            router_creator.key().as_ref()
        ],
        bump = name_router_account.bump
    )]
    pub name_router_account: Account<'info, NameRouter>,

    #[account(
        address = name_router_account.router_creator
    )]
    pub router_creator: SystemAccount<'info>,

    #[account(
        init,
        space = Repository::size(),
        payer = repository_creator,
        seeds = [
            b"repository",
            id.as_bytes(),
            repository_creator.key().as_ref(),
        ],
        bump
    )]
    pub repository_account: Account<'info, Repository>,
    #[account(
        init,
        payer = repository_creator,
        space = VestingSchedule::size(default_schedule.number_of_schedules.into()),
        seeds = [
            b"vesting",
            repository_account.key().as_ref(),
        ],
        bump
    )]
    pub vesting_account: Option<Account<'info, VestingSchedule>>,
    #[account(mut)]
    ///CHECK: The account checks are done in function, unchecked as it might not exist and will be created in that case
    pub vesting_token_account: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    ///CHECK: The account checks are done in function, unchecked as it might not exist and will be created in that case
    pub repository_creator_token_account: Option<UncheckedAccount<'info>>,
    #[account(
        seeds = [
            b"isGodReal?",
            b"DoULoveMe?",
            b"SweetChick"
        ],
        bump = default_schedule.bump,
    )]
    pub default_schedule: Box<Account<'info, DefaultVestingSchedule>>,
    /// CHECK: We're about to create this with Metaplex
    #[account(mut)]
    pub metadata: Option<UncheckedAccount<'info>>,
    #[account(
        init_if_needed,
        payer = repository_creator,
        mint::authority = rewards_mint,
        mint::decimals = 0,
        seeds = [b"Miners",
        b"MinerC",
        repository_account.key().as_ref()],
        bump
    )]
    pub rewards_mint: Option<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_metadata_program: Program<'info, Metadata>,
}

pub fn handler(
    ctx: Context<CreateRepository>,
    id: String,
    description: String,
    uri: String,
    token_name: Box<Option<String>>,
    token_symbol: Box<Option<String>>,
    token_metadata_uri: Box<Option<String>>,
) -> Result<()> {
    let repository_account = &mut ctx.accounts.repository_account;
    let name_router_account = &ctx.accounts.name_router_account;
    let repository_verified_user = &ctx.accounts.repository_verified_user;
    let rewards_mint = &ctx.accounts.rewards_mint;
    let vesting_account = &mut ctx.accounts.vesting_account;
    let vesting_token_account = &mut ctx.accounts.vesting_token_account;
    let repository_creator = &mut ctx.accounts.repository_creator;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let repository_creator_token_account = &ctx.accounts.repository_creator_token_account;
    let default_schedule = &ctx.accounts.default_schedule;
    let metadata = &mut ctx.accounts.metadata;

    //fills repository account data
    repository_account.bump = *ctx.bumps.get("repository_account").unwrap();
    repository_account.name_router = name_router_account.key();
    repository_account.repository_creator = repository_verified_user.user_pubkey.key();
    repository_account.id = id;
    repository_account.description = description;
    repository_account.uri = uri;
    repository_account.issue_index = 0;

    let repository_account_key = repository_account.key();
    let mut rewards_mint_key: Option<Pubkey> = None;
    let mut vesting_schedule_key: Option<Pubkey> = None;
    match rewards_mint {
        Some(rewards_mint) => {
            rewards_mint_key = Some(rewards_mint.key());
            match vesting_account {
                Some(vesting_account) => {
                    vesting_schedule_key = Some(vesting_account.key());
                    match vesting_token_account {
                        Some(vesting_token_account) => {
                            match repository_creator_token_account {
                                Some(repository_creator_token_account) => {
                                    match metadata {
                                        Some(metadata) => {
                                            require!(
                                                metadata.key().eq(&(find_metadata_account(
                                                    &rewards_mint.key()
                                                )
                                                .0)),
                                                DefiOSError::IncorrectMetadataAccount
                                            );
                                            // Unpack token data
                                            let name: String;
                                            let uri: String;
                                            let symbol: String;
                                            match *token_name {
                                                Some(ref token_name) => {
                                                    name = token_name.clone().to_string();
                                                }
                                                None => {
                                                    name = "".to_string();
                                                }
                                            };
                                            match *token_symbol {
                                                Some(ref token_symbol) => {
                                                    symbol = token_symbol.clone().to_string();
                                                }
                                                None => {
                                                    symbol = "".to_string();
                                                }
                                            };
                                            match *token_metadata_uri {
                                                Some(ref token_metadata_uri) => {
                                                    uri = token_metadata_uri.clone().to_string();
                                                }
                                                None => {
                                                    uri = "".to_string();
                                                }
                                            };

                                            // Create vesting token account if empty
                                            if vesting_token_account.data_is_empty() {
                                                create(CpiContext::new(
                                                    associated_token_program.to_account_info(),
                                                    Create {
                                                        payer: repository_creator.to_account_info(),
                                                        associated_token: vesting_token_account
                                                            .to_account_info(),
                                                        authority: vesting_account
                                                            .to_account_info(),
                                                        mint: rewards_mint.to_account_info(),
                                                        system_program: system_program
                                                            .to_account_info(),
                                                        token_program: token_program
                                                            .to_account_info(),
                                                    },
                                                ))?;
                                            }

                                            // Create repository token account if empty
                                            if repository_creator_token_account.data_is_empty() {
                                                create(CpiContext::new(
                                                    associated_token_program.to_account_info(),
                                                    Create {
                                                        payer: repository_creator.to_account_info(),
                                                        associated_token:
                                                            repository_creator_token_account
                                                                .to_account_info(),
                                                        authority: repository_creator
                                                            .to_account_info(),
                                                        mint: rewards_mint.to_account_info(),
                                                        system_program: system_program
                                                            .to_account_info(),
                                                        token_program: token_program
                                                            .to_account_info(),
                                                    },
                                                ))?;
                                            }

                                            // Add checks to ensure token vesting accounts are correct
                                            let expected_vesting_token_account =
                                                get_associated_token_address(
                                                    &vesting_account.key(),
                                                    &rewards_mint.key(),
                                                );
                                            let expected_repository_token_pool_account =
                                                get_associated_token_address(
                                                    &repository_creator.key(),
                                                    &rewards_mint.key(),
                                                );
                                            require!(
                                                expected_vesting_token_account.eq(&vesting_token_account.key())
                                                    && expected_repository_token_pool_account.eq(&repository_creator_token_account.key()),
                                                DefiOSError::TokenAccountMismatch
                                            );

                                            let bump = *ctx.bumps.get("rewards_mint").unwrap();

                                            let signer_seeds: &[&[&[u8]]] = &[&[
                                                b"Miners",
                                                b"MinerC",
                                                repository_account_key.as_ref(),
                                                &[bump],
                                            ]];

                                            token::mint_to(
                                                CpiContext::new_with_signer(
                                                    ctx.accounts.token_program.to_account_info(),
                                                    token::MintTo {
                                                        mint: rewards_mint.to_account_info(),
                                                        to: vesting_token_account.to_account_info(),
                                                        authority: rewards_mint.to_account_info(),
                                                    },
                                                    signer_seeds,
                                                ),
                                                default_schedule.per_vesting_amount
                                                    * (default_schedule.number_of_schedules as u64),
                                            )?;

                                            invoke_signed(
                                                &token_instruction::create_metadata_accounts_v3(
                                                    mpl_token_metadata::id(), // program_id
                                                    metadata.key(),           // metadata_account
                                                    rewards_mint.key(),       // mint
                                                    rewards_mint.key(),       // mint_authority
                                                    repository_creator.key(), // payer
                                                    rewards_mint.key(),       // update_authority
                                                    name,                     // name
                                                    symbol,                   // symbol
                                                    uri,                      // uri
                                                    None,                     // creators
                                                    0,     // seller_fee_basis_points
                                                    false, // update_authority_is_signer
                                                    true,  // is_mutable
                                                    None,  // collection
                                                    None,  // uses
                                                    None,  // collection_details
                                                ),
                                                &[
                                                    metadata.to_account_info(),           // Metadata account
                                                    rewards_mint.to_account_info(), // Mint account
                                                    rewards_mint.to_account_info(), // Mint authority
                                                    repository_creator.to_account_info(), // Payer
                                                    rewards_mint.to_account_info(), // Update authority
                                                    system_program.to_account_info(), // System program
                                                ],
                                                signer_seeds,
                                            )?;

                                            // Add data to token vesting account
                                            vesting_account.bump =
                                                *ctx.bumps.get("vesting_account").unwrap();
                                            vesting_account.destination_address =
                                                repository_creator_token_account.key();
                                            vesting_account.mint_address = rewards_mint.key();
                                            vesting_account.schedules = vec![];

                                            // Add schedules to vesting
                                            let mut release_time = u64::from_ne_bytes(
                                                Clock::get()?.unix_timestamp.to_ne_bytes(),
                                            );
                                            for _i in 0..default_schedule.number_of_schedules {
                                                vesting_account.schedules.push(Schedule {
                                                    release_time: release_time,
                                                    amount: default_schedule.per_vesting_amount,
                                                });
                                                release_time += default_schedule.unix_change;
                                            }
                                        }
                                        None => {}
                                    }
                                }
                                None => {}
                            }
                        }
                        None => {}
                    }
                }
                None => {}
            }
        }
        None => {}
    }

    //add vestifn schedule and repository mint key to repository
    repository_account.vesting_schedule = vesting_schedule_key;
    repository_account.rewards_mint = rewards_mint_key;
    //emits event of repository created
    emit!(RepositoryCreated {
        repository_creator: repository_verified_user.user_pubkey.key(),
        repository_account: repository_account.key(),
        uri: repository_account.uri.clone(),
        rewards_mint: rewards_mint_key,
        id: repository_account.id.clone(),
        description: repository_account.description.clone(),
        token_name: *token_name,
        token_symbol: *token_symbol,
        token_metadata_uri: *token_metadata_uri,
        vesting_account: vesting_schedule_key
    });
    Ok(())
}
