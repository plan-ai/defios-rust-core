use crate::helper::find_metadata_account;
use crate::{
    constants::{DEFAULT_MINT_DECIMALS, RELEASE_TIME, TOKEN_VEST_AMOUNT, VESTING_NUMBER},
    error::DefiOSError,
    event::RepositoryCreated,
    state::{Repository, Schedule, VerifiedUser, VestingSchedule},
};
use anchor_lang::prelude::*;
use anchor_spl::metadata::mpl_token_metadata::types::DataV2;
use anchor_spl::{
    associated_token::{create, get_associated_token_address, AssociatedToken, Create},
    metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata},
    token,
    token::{Mint, Token},
};

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
            repository_verified_user.name_router.as_ref()
        ],
        bump = repository_verified_user.bump
    )]
    pub repository_verified_user: Account<'info, VerifiedUser>,

    #[account(
        init,
        space = 8+Repository::INIT_SPACE,
        payer = repository_creator,
        seeds = [
            b"repository",
            id.as_bytes(),
            repository_creator.key().as_ref(),
        ],
        bump
    )]
    pub repository_account: Box<Account<'info, Repository>>,
    #[account(
        init,
        payer = repository_creator,
        space = 8+VestingSchedule::INIT_SPACE,
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
    /// CHECK: We're about to create this with Metaplex
    #[account(mut)]
    pub metadata: Option<UncheckedAccount<'info>>,
    #[account(
        init,
        payer = repository_creator,
        mint::authority = rewards_mint,
        mint::decimals = DEFAULT_MINT_DECIMALS,
        seeds = [b"Miners",
        b"MinerC",
        repository_account.key().as_ref()],
        bump
    )]
    pub rewards_mint: Option<Account<'info, Mint>>,
    pub imported_mint: Option<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
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
    let repository_verified_user = &ctx.accounts.repository_verified_user;
    let rewards_mint = &ctx.accounts.rewards_mint;
    let vesting_account = &mut ctx.accounts.vesting_account;
    let vesting_token_account = &mut ctx.accounts.vesting_token_account;
    let repository_creator = &mut ctx.accounts.repository_creator;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;
    let associated_token_program = &ctx.accounts.associated_token_program;
    let repository_creator_token_account = &ctx.accounts.repository_creator_token_account;
    let metadata = &mut ctx.accounts.metadata;
    let imported_mint = &ctx.accounts.imported_mint;
    let rent = &ctx.accounts.rent;

    //fills repository account data
    repository_account.bump = ctx.bumps.repository_account;
    repository_account.repository_creator = repository_verified_user.user_pubkey.key();
    repository_account.id = id;
    repository_account.description = description;
    repository_account.uri = uri;
    repository_account.issue_index = 0;

    let repository_account_key = repository_account.key();
    let mut rewards_mint_key: Option<Pubkey> = None;
    let mut vesting_schedule_key: Option<Pubkey> = None;
    let mut token_imported: bool = false;
    if let (
        Some(rewards_mint),
        Some(vesting_account),
        Some(vesting_token_account),
        Some(repository_creator_token_account),
        Some(metadata),
    ) = (
        rewards_mint,
        vesting_account,
        vesting_token_account,
        repository_creator_token_account,
        metadata,
    ) {
        rewards_mint_key = Some(rewards_mint.key());
        vesting_schedule_key = Some(vesting_account.key());

        require!(
            metadata
                .key()
                .eq(&(find_metadata_account(&rewards_mint.key()).0)),
            DefiOSError::IncorrectMetadataAccount
        );

        // Unpack token data
        let name: String;
        let symbol: String;
        let uri: String;

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
                    associated_token: vesting_token_account.to_account_info(),
                    authority: vesting_account.to_account_info(),
                    mint: rewards_mint.to_account_info(),
                    system_program: system_program.to_account_info(),
                    token_program: token_program.to_account_info(),
                },
            ))?;
        }

        // Create repository token account if empty
        if repository_creator_token_account.data_is_empty() {
            create(CpiContext::new(
                associated_token_program.to_account_info(),
                Create {
                    payer: repository_creator.to_account_info(),
                    associated_token: repository_creator_token_account.to_account_info(),
                    authority: repository_creator.to_account_info(),
                    mint: rewards_mint.to_account_info(),
                    system_program: system_program.to_account_info(),
                    token_program: token_program.to_account_info(),
                },
            ))?;
        }
        // Add checks to ensure token vesting accounts are correct
        let expected_vesting_token_account =
            get_associated_token_address(&vesting_account.key(), &rewards_mint.key());
        let expected_repository_token_pool_account =
            get_associated_token_address(&repository_creator.key(), &rewards_mint.key());
        require!(
            expected_vesting_token_account.eq(&vesting_token_account.key())
                && expected_repository_token_pool_account
                    .eq(&repository_creator_token_account.key()),
            DefiOSError::TokenAccountMismatch
        );

        let bump = ctx.bumps.rewards_mint;
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
            VESTING_NUMBER * TOKEN_VEST_AMOUNT * u64::pow(10, rewards_mint.decimals.into()),
        )?;

        // On-chain token metadata for the mint
        let data_v2 = DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let cpi_ctx = CpiContext::new_with_signer(
            rewards_mint.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: metadata.to_account_info(),
                mint: rewards_mint.to_account_info(),
                mint_authority: rewards_mint.to_account_info(),
                update_authority: rewards_mint.to_account_info(),
                payer: repository_creator.to_account_info(),
                system_program: system_program.to_account_info(),
                rent: rent.to_account_info(),
            },
            signer_seeds,
        );

        create_metadata_accounts_v3(cpi_ctx, data_v2, true, true, None)?;

        // Add data to token vesting account
        vesting_account.bump = ctx.bumps.vesting_account;
        vesting_account.destination_address = repository_creator_token_account.key();
        vesting_account.mint_address = rewards_mint.key();
        vesting_account.schedules = vec![];

        // Add schedules to vesting
        let mut release_time = u64::from_ne_bytes(Clock::get()?.unix_timestamp.to_ne_bytes());
        for _i in 0..VESTING_NUMBER {
            vesting_account.schedules.push(Schedule {
                release_time,
                amount: TOKEN_VEST_AMOUNT * u64::pow(10, rewards_mint.decimals.into()),
            });
            release_time += RELEASE_TIME;
        }
    } else {
        if let Some(imported_mint) = imported_mint {
            rewards_mint_key = Some(imported_mint.key());
            token_imported = true;
        }
    }

    //add vesting schedule and repository mint key to repository
    repository_account.vesting_schedule = vesting_schedule_key;
    repository_account.repo_token = rewards_mint_key.unwrap();

    repository_account.new_token = !token_imported;
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
        vesting_account: vesting_schedule_key,
        token_imported: token_imported
    });
    Ok(())
}
