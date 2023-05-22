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
    token,
    token::{Mint, Token},
};

#[derive(Accounts)]
#[instruction(name: String)]
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
            name.as_bytes(),
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
            rewards_mint.key().as_ref(),
            repository_account.key().as_ref(),
        ],
        bump
    )]
    pub vesting_account: Account<'info, VestingSchedule>,
    #[account(mut)]
    ///CHECK: The account checks are done in function, unchecked as it might not exist and will be created in that case
    pub vesting_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    ///CHECK: The account checks are done in function, unchecked as it might not exist and will be created in that case
    pub repository_creator_token_account: UncheckedAccount<'info>,
    #[account(
        seeds = [
            b"isGodReal?",
            b"DoULoveMe?",
            b"SweetChick"
        ],
        bump = default_schedule.bump,
    )]
    pub default_schedule: Account<'info, DefaultVestingSchedule>,
    #[account(
        init,
        payer = repository_creator,
        mint::authority = rewards_mint,
        mint::decimals = 9,
        seeds = [b"Miners",
        b"MinerC",
        repository_account.key().as_ref()],
        bump
    )]
    pub rewards_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateRepository>,
    name: String,
    description: String,
    uri: String,
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

    //logs repository and spl token creation
    msg!(
        "Creating repository of name: {} Repository address: {} Rewards mint: {}",
        &name,
        repository_account.key().to_string(),
        rewards_mint.key().to_string(),
    );

    //fills repository account data
    repository_account.bump = *ctx.bumps.get("repository_account").unwrap();
    repository_account.name_router = name_router_account.key();
    repository_account.repository_creator = repository_verified_user.user_pubkey.key();
    repository_account.rewards_mint = rewards_mint.key();
    repository_account.name = name;
    repository_account.description = description;
    repository_account.uri = uri;
    repository_account.issue_index = 0;

    //emits event of repository created
    emit!(RepositoryCreated {
        repository_creator: repository_verified_user.user_pubkey.key(),
        repository_account: repository_account.key(),
        uri: repository_account.uri.clone(),
        rewards_mint: rewards_mint.key(),
        name: repository_account.name.clone(),
        description: repository_account.description.clone()
    });

    //creates vesting token account if empty
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

    //creates repository token account if empty
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

    let repository_account_key = repository_account.key();
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
        default_schedule.per_vesting_amount * (default_schedule.number_of_schedules as u64),
    )?;

    //add checks for making sure token vesting accounts are correct
    let expected_vesting_token_account =
        get_associated_token_address(&vesting_account.key(), &rewards_mint.key());
    let expected_repository_token_pool_account =
        get_associated_token_address(&repository_creator.key(), &rewards_mint.key());
    require!(
        expected_vesting_token_account.eq(&vesting_token_account.key())
            && expected_repository_token_pool_account.eq(&repository_creator_token_account.key()),
        DefiOSError::TokenAccountMismatch
    );

    //add data to token vesting account
    vesting_account.bump = *ctx.bumps.get("vesting_account").unwrap();
    vesting_account.destination_address = repository_creator_token_account.key();
    vesting_account.mint_address = rewards_mint.key();
    vesting_account.schedules = vec![];

    //adding schedules to vesting
    let mut release_time = u64::from_ne_bytes(Clock::get()?.unix_timestamp.to_ne_bytes());
    for _i in 0..default_schedule.number_of_schedules {
        vesting_account.schedules.push(Schedule {
            release_time: release_time,
            amount: default_schedule.per_vesting_amount,
        });
        release_time += default_schedule.unix_change;
    }

    //add vestifn schedule to repository
    repository_account.vesting_schedule = vesting_account.key();
    Ok(())
}
