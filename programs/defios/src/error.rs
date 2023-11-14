use anchor_lang::error_code;

#[error_code]
pub enum DefiOSError {
    // 6000
    #[msg("Invalid Signature")]
    SignatureVerificationFailed,

    // 6001
    #[msg("User not verified")]
    UnauthorizedUser,

    // 6002
    #[msg("Invalid Name Router")]
    InvalidNameRouter,

    // 6003
    #[msg("Token account mismatch")]
    TokenAccountMismatch,

    // 6004
    #[msg("Insufficient funds for staking")]
    InsufficientStakingFunds,

    // 6005
    #[msg("Cannot stake/unstake for a closed issue")]
    IssueClosedAlready,

    // 6006
    #[msg("Commit hashes do not match for reward eligibility")]
    HashesMismatch,

    // 6007
    #[msg("Tokens Already Claimed")]
    AlreadyClaimed,

    // 6008
    #[msg("Cannot stake/unstake on a closed objective")]
    ObjectiveClosedAlready,

    // 6009
    #[msg("Parent was not mentioned to which objective is to be added")]
    NoParentEntered,

    //6010
    #[msg("Roadmap end time before roadmap creation time")]
    RoadmapInvalidEndTime,

    //6011
    #[msg("Can not add PR of somebody else's commits")]
    UnauthorizedPR,

    // 6012
    #[msg("Math overflow")]
    MathOverflow,

    // 6013
    #[msg("Token Mint mismatch")]
    MintMismatch,

    // 6014
    #[msg("Vesting contract has not reached release time")]
    VestingNotReachedRelease,

    //6015
    #[msg("Pull request not yet accepted")]
    PullRequestNotYetAccepted,

    //6016
    #[msg("You are not authorized to merge this pull request")]
    CanNotMergePullRequest,

    //6017
    #[msg("Unauthorized smart contract Action")]
    UnauthorizedActionAttempted,

    //6018
    #[msg("No money was staked on this issue, Still thanks for the support to the community")]
    NoMoneyStakedOnIssue,

    // 6019
    #[msg("Insufficient funds")]
    InsufficientFunds,

    //6020
    #[msg("Incorrect Inputs for buy/sell given")]
    IncorrectMaths,

    //6021
    #[msg("Incorrect Metadata account provided")]
    IncorrectMetadataAccount,

    // 6022
    #[msg("Cannot vote on a closed issue")]
    PullRequestVotingClosedAlready,

    //6023
    #[msg("Unauthorized objective addition")]
    CantAddObjectiveToSomebodiesRoadmap,

    //6024
    #[msg("Cant enter time below 0")]
    CantEnterTimeBelowZero,

    //6025
    #[msg("No PR on this issue to vote on")]
    NoPRFound,

    //6026
    #[msg("Voting period has ended")]
    VotingPeriodEnded,

    //6027
    #[msg("Can't unstake after voting")]
    CantUnstakeAfterVoting,

    //6028
    #[msg("Either need to import to create a token")]
    NoRepoTokenSpecified,

    //6029
    #[msg("Pull request account not sent to auto charge votes")]
    PullRequestAutoUpdate,

    //6030
    #[msg("Invalid parent enetered")]
    InvalidObjectiveParent,

    //6031
    #[msg("Not allowed to change repo token")]
    RepoTokenChangeRejected,

    //6032
    #[msg("Not enough votes on PR to merge")]
    NotEnoughVotesForIssueMerge,
}
