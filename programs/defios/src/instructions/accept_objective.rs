use crate::constants::VOTING_END;
use crate::error::DefiOSError;
use crate::event::ObjectiveAccepted;
use crate::state::{Objective, ObjectiveProposal, Repository};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AcceptObjective<'info> {
    pub initiator: Signer<'info>,
    #[account(
        mut,
        constraint = objective.completed_at == None,
        constraint = objective.total_dispersed_grant == objective.total_grant,
        constraint = objective.objective_repository == repository_account.key()
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
    #[account(
        mut,
        seeds = [
            b"repository",
            repository_account.id.as_bytes(),
            repository_account.repository_creator.key().as_ref(),
        ],
        bump = repository_account.bump
    )]
    pub repository_account: Account<'info, Repository>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AcceptObjective>) -> Result<()> {
    let objective_proposal = &mut ctx.accounts.objective_proposal;
    let objective = &mut ctx.accounts.objective;
    let repository_account = &mut ctx.accounts.repository_account;

    let current_time = Clock::get()?.unix_timestamp;
    require!(
        current_time - objective_proposal.proposed_at > VOTING_END,
        DefiOSError::VotingPeriodOnGoing
    );

    objective.completed_at = Some(current_time);
    repository_account.objectives_open -= 1;
    emit!(ObjectiveAccepted {
        objective: objective.key(),
        objective_proposal: objective_proposal.key()
    });

    Ok(())
}
