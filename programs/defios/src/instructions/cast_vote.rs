use anchor_lang::prelude::*;
use crate::state::{VoteCasted, Vote,Issue};

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account]
    pub voter: Signer<'info>,
    #[account]
    pub issue_account Account<'info,Issue>
    #[account(
        init,
        payer = voter,
        space = Vote::size(),
        seeds = [
            b"votecasted",
            voter.key().as_ref(),
            issue_account.key().as_ref()
        ],
        bump
    )]
    pub vote_metadata_store Account<'info,Vote>
    #[account]
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CastVote>) -> Result<()> {
    let voter =  &ctx.accounts.voter;
    let vote_metadata_store = &mut ctx.accounts.vote_metadata_store;
    let issue_account = &ctx.accounts.issue_account;

    msg!(
        "{} voted on {}",
        voter.key(),
        issue_account.key()
    );


    vote_metadata_store.issue_pub_key = issue_account.key();
    vote_metadata_store.voted_by = voter.key();
    
    emit!(VoteCasted {
            issue_pub_key: issue_account.key(),
            voted_by: voter.key()
    });
    
    Ok(())
}