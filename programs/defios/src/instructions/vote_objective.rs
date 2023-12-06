use crate::constants::VOTING_END;
use crate::error::DefiOSError;
use crate::event::ObjectiveProposalVoted;
use crate::state::{Grantee, Objective, ObjectiveProposal};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct VoteObjective<'info> {
    pub voter: Signer<'info>,
    #[account(
        constraint = objective.completed_at == None
    )]
    pub objective: Account<'info, Objective>,
    #[account(
        mut,
        seeds = [
            voter.key().as_ref(),
            objective.objective_repository.as_ref(),
            grant_account.objective.as_ref(),
        ],
        bump = grant_account.bump
    )]
    pub grant_account: Account<'info, Grantee>,
    #[account(
        mut,
        seeds = [
            b"objective_proposal",
            objective.key().as_ref(),
            proposal.proposal_id.as_bytes()
        ],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, ObjectiveProposal>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<VoteObjective>, positive: bool) -> Result<()> {
    let voter = &ctx.accounts.voter;
    let grant_account = &mut ctx.accounts.grant_account;
    let proposal = &mut ctx.accounts.proposal;
    let objective = &ctx.accounts.objective;

    let current_time = u64::from_ne_bytes(Clock::get()?.unix_timestamp.to_ne_bytes());
    require!(
        current_time - proposal.proposed_at <= VOTING_END,
        DefiOSError::VotingPeriodEnded
    );

    let capacity = grant_account.staked_amount - grant_account.voted_amount;
    if positive {
        proposal.vote_amount += capacity;
    } else {
        proposal.deny_amount += capacity;
    };

    grant_account.voted_amount += capacity;

    emit!(ObjectiveProposalVoted {
        voter: voter.key(),
        objective: objective.key(),
        objective_proposal: proposal.key(),
        vote_amount: grant_account.staked_amount,
    });

    Ok(())
}
