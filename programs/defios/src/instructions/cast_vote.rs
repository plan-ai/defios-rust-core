use anchor_lang::prelude::*;
use crate::state::{VoteCasted, Vote,PullRequest};

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(mut)]
    pub pr_account: Account<'info,PullRequest>,
    #[account(
        init,
        payer = voter,
        space = Vote::size(),
        seeds = [
            b"votecasted",
            voter.key().as_ref(),
            pr_account.key().as_ref()
        ],
        bump
    )]
    pub vote_metadata_store: Account<'info,Vote>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CastVote>) -> Result<()> {
    let voter =  &ctx.accounts.voter;
    let vote_metadata_store = &mut ctx.accounts.vote_metadata_store;
    let pr_account = &ctx.accounts.pr_account;

    msg!(
        "{} voted on {}",
        voter.key(),
        pr_account.key()
    );


    vote_metadata_store.pr_pub_key = pr_account.key();
    vote_metadata_store.voted_by = voter.key();
    
    emit!(VoteCasted {
            pr_pub_key: pr_account.key(),
            voted_by: voter.key()
    });
    
    Ok(())
}