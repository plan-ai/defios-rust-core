use crate::constants::AUTHORIZED_PUBLIC_KEY;
use crate::error::ApplicationError;
use crate::state::complaint::{Complaint, ComplaintAccepted};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AcceptComplaint<'info> {
    #[account(mut,address = AUTHORIZED_PUBLIC_KEY@ApplicationError::NotAuthroizedToAcceptComplaint)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub complaint: Account<'info, Complaint>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AcceptComplaint>) -> Result<()> {
    let complaint = &mut ctx.accounts.complaint;

    complaint.accepted = true;

    emit!(ComplaintAccepted {
        complaint: complaint.key()
    });

    Ok(())
}
