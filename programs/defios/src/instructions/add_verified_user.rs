use crate::{
    error::DefiOSError,
    event::VerifiedUserAdded,
    state::{NameRouter, VerifiedUser},
};
use anchor_lang::prelude::*;
use solana_program::{
    ed25519_program::ID as ED25519ProgramID,
    instruction::Instruction,
    sysvar::instructions::{load_instruction_at_checked, ID as SysvarInstructionsID},
};

#[derive(Accounts)]
#[instruction(user_name: String, user_pubkey: Pubkey)]
pub struct AddVerifiedUser<'info> {
    #[account(
        mut,
        address = name_router_account.router_creator
    )]
    pub router_creator: Signer<'info>,

    #[account(
        mut,
        seeds = [
            name_router_account.signing_domain.as_bytes(),
            name_router_account.signature_version.to_string().as_bytes(),
            router_creator.key().as_ref()
        ],
        bump = name_router_account.bump
    )]
    pub name_router_account: Account<'info, NameRouter>,

    #[account(
        init,
        payer = router_creator,
        space = 8+VerifiedUser::INIT_SPACE,
        seeds = [
            user_name.as_bytes(),
            user_pubkey.as_ref(),
            name_router_account.key().as_ref(),
        ],
        bump,
    )]
    pub verified_user_account: Account<'info, VerifiedUser>,

    /// CHECK: Address check done
    #[account(address = SysvarInstructionsID)]
    pub sysvar_instructions: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AddVerifiedUser>,
    user_name: String,
    user_pubkey: Pubkey,
    msg: Vec<u8>,
    sig: [u8; 64],
) -> Result<()> {
    let sysvar_instructions = &ctx.accounts.sysvar_instructions;
    // Checking ED25519 instruction
    let ed25519_create_ix = load_instruction_at_checked(0, sysvar_instructions)?;
    let router_creator = &ctx.accounts.router_creator.key();

    verify_signature(&ed25519_create_ix, &router_creator.to_bytes(), &msg, &sig)?;

    let name_router_account = &mut ctx.accounts.name_router_account;

    let verified_user_account = &mut ctx.accounts.verified_user_account;

    name_router_account.total_verified_users =
        name_router_account.total_verified_users.saturating_add(1);

    verified_user_account.bump = *ctx.bumps.get("verified_user_account").unwrap();
    verified_user_account.name_router = name_router_account.key();
    verified_user_account.user_name = user_name;
    verified_user_account.user_pubkey = user_pubkey;
    emit!(VerifiedUserAdded {
        router_creator: router_creator.key(),
        name_router_account: name_router_account.key(),
        verified_user_account: verified_user_account.key(),
        user_name: verified_user_account.user_name.clone(),
        user_pubkey: user_pubkey
    });
    Ok(())
}

pub fn verify_signature(
    create_ix: &Instruction,
    user_pubkey: &[u8; 32],
    msg: &Vec<u8>,
    sig: &[u8; 64],
) -> Result<()> {
    if create_ix.program_id != ED25519ProgramID
        || !create_ix.accounts.is_empty()
        || create_ix.data.len() != (16 + 64 + 32 + msg.len())
    {
        return err!(DefiOSError::SignatureVerificationFailed);
    }

    let create_ix_data = &create_ix.data;

    // Dissecting ix data
    let num_signatures = &[create_ix_data[0]]; // u8
    let padding = &[create_ix_data[1]]; // u8
    let signature_offset = &create_ix_data[2..4]; // u16
    let signature_instruction_index = &create_ix_data[4..6]; // u16
    let public_key_offset = &create_ix_data[6..8]; // u16
    let public_key_instruction_index = &create_ix_data[8..10]; // u16
    let message_data_offset = &create_ix_data[10..12]; // u16
    let message_data_size = &create_ix_data[12..14]; // u16
    let message_instruction_index = &create_ix_data[14..16]; // u16

    let data_pubkey = &create_ix_data[16..48]; // Pubkey (32 bytes)
    let data_sig = &create_ix_data[48..112]; // Signature (64 bytes)
    let data_msg = &create_ix_data[112..]; // Message (Dynamic)

    // Checking ix data
    let exp_public_key_offset: u16 = 16; // Span of state (line 51 - 59)
    let exp_signature_offset: u16 = exp_public_key_offset + user_pubkey.len() as u16;
    let exp_message_data_offset: u16 = exp_signature_offset + sig.len() as u16;
    let exp_num_signatures: u8 = 1;
    let exp_message_data_size: u16 = msg.len().try_into().unwrap();

    if num_signatures != &exp_num_signatures.to_le_bytes()
        || padding != &[0]
        || signature_offset != exp_signature_offset.to_le_bytes()
        || signature_instruction_index != u16::MAX.to_le_bytes()
        || public_key_offset != exp_public_key_offset.to_le_bytes()
        || public_key_instruction_index != u16::MAX.to_le_bytes()
        || message_data_offset != exp_message_data_offset.to_le_bytes()
        || message_data_size != exp_message_data_size.to_le_bytes()
        || message_instruction_index != u16::MAX.to_le_bytes()
    {
        return err!(DefiOSError::SignatureVerificationFailed);
    }

    // Checking pubkey, signature, message
    if data_pubkey != user_pubkey || data_msg != msg || data_sig != sig {
        return err!(DefiOSError::SignatureVerificationFailed);
    }

    Ok(())
}
