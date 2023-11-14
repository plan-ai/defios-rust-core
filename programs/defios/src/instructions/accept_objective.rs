use crate::event::ObjectiveAccepted;
use crate::state::{Objective, ObjectiveProposal};
use crate::constants::VOTING_END;
use crate::error::DefiOSError;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AcceptObjective<'info> {
    pub initiator: Signer<'info>,
    #[account(
        constraint = objective.completed_at == None,
        constraint = objective.total_dispersed_grant == objective.total_grant
    )]
    pub objective: Account<'info, Objective>,
    #[account(
        mut,
        seeds = [
            b"objective_proposal",
            objective.key().as_ref(),
            objective_proposal.proposal_id.as_bytes()
        ],
        bump = objective_proposal.bump,
        constraint = objective_proposal.vote_amount>objective_proposal.deny_amount
    )]
    pub objective_proposal: Account<'info, ObjectiveProposal>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AcceptObjective>) -> Result<()> {
    let objective_proposal = &mut ctx.accounts.objective_proposal;
    let objective = &mut ctx.accounts.objective;

    let current_time = u64::from_ne_bytes(Clock::get()?.unix_timestamp.to_ne_bytes());
    require!(
        current_time - objective_proposal.proposed_at > VOTING_END,
        DefiOSError::VotingPeriodOnGoing
    );

    objective.completed_at = Some(current_time);

    emit!(ObjectiveAccepted {
        objective: objective.key(),
        objective_proposal: objective_proposal.key()
    });

    Ok(())
}
