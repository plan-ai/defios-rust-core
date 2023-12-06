use crate::event::ObjectiveProposalCreated;
use crate::state::{Objective, ObjectiveProposal};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(proposal_id: String)]
pub struct CreateObjectiveProposal<'info> {
    #[account(mut)]
    pub proposee: Signer<'info>,
    #[account(
        constraint = objective.completed_at == None
    )]
    pub objective: Account<'info, Objective>,
    #[account(
        init,
        payer = proposee,
        space = 8 + ObjectiveProposal::INIT_SPACE,
        seeds = [
            b"objective_proposal",
            objective.key().as_ref(),
            proposal_id.as_bytes()
        ],
        bump
    )]
    pub objective_proposal: Account<'info, ObjectiveProposal>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateObjectiveProposal>,
    proposal_id: String,
    objective_proposal_url: String,
) -> Result<()> {
    let objective_proposal = &mut ctx.accounts.objective_proposal;
    let proposee = &ctx.accounts.proposee;
    let objective = &ctx.accounts.objective;

    objective_proposal.bump = ctx.bumps.objective_proposal;
    objective_proposal.proposal_id = proposal_id;
    objective_proposal.proposee = proposee.key();
    objective_proposal.objective = objective.key();
    objective_proposal.proposal_metadata_uri = objective_proposal_url.clone();
    objective_proposal.proposed_at = Clock::get()?.unix_timestamp;
    objective_proposal.vote_amount = 0;
    objective_proposal.deny_amount = 0;

    emit!(ObjectiveProposalCreated {
        objective: objective.key(),
        proposee: proposee.key(),
        proposed_time: objective_proposal.proposed_at,
        objective_proposal_url: objective_proposal_url
    });

    Ok(())
}
