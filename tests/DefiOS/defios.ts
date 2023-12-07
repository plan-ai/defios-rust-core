import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Defios } from "../../target/types/defios";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
  createMint,
  transfer,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { Metaplex } from "@metaplex-foundation/js";
import { rpcConfig } from "../test_config";
import * as constant from "../constants";
import {
  create_keypair,
  create_name_router,
  create_spl_token,
  create_verified_user,
  get_metadata_account,
  get_pda_from_seeds,
} from "./helper";

describe("defios", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  //testing defios workspace here
  const program = anchor.workspace.Defios as Program<Defios>;
  const {
    provider: { connection },
  } = program;
  const { web3 } = anchor;
  const metaplex = Metaplex.make(connection);

  //main testsuite
  //creating a name router
  let global = {};
  it("Creates a name router!", async () => {
    let [routerCreatorKeypair, nameRouterAccount] = await create_name_router();
    //get data related to name router pda
    const {
      routerCreator,
      signatureVersion: SignatureVersion,
      signingDomain,
      bump,
      totalVerifiedUsers,
    } = await program.account.nameRouter.fetch(nameRouterAccount);
    global.nameRouterAccount = nameRouterAccount;
    global.routerCreatorKeypair = routerCreatorKeypair;
  });

  it("Adds a verified user", async () => {
    const [verifiedUserAccount] = await create_verified_user(
      global.routerCreatorKeypair,
      global.nameRouterAccount,
      constant.userPubkey
    );
    global.verifiedUserAccount = verifiedUserAccount;
  });

  it("Creates a repository with new spl token", async () => {
    //generates key pairs and airdrops solana to them
    let [nameRouterAccount, routerCreatorKeypair] = [
      global.nameRouterAccount,
      global.routerCreatorKeypair,
    ];
    const repositoryCreator = await create_keypair();
    const [repositoryCreatorVerifiedAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      repositoryCreator.publicKey
    );

    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
    ] = await create_spl_token(repositoryCreator);

    const metadataAddress = await get_metadata_account(mintKeypair);

    await program.methods
      .createRepository(
        constant.repositoryId,
        constant.repositoryTitle,
        constant.repositoryUri,
        constant.tokenName,
        constant.tokenimage,
        constant.tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: repositoryCreatorVerifiedAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        metadata: metadataAddress,
        tokenMetadataProgram: constant.TOKEN_METADATA_PROGRAM_ID,
        importedMint: null,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
    global.repositoryCreator = repositoryCreator;
    global.repositoryCreatorVerifiedAccount = repositoryCreatorVerifiedAccount;
    global.repositoryAccount = repositoryAccount;
    global.mintKeypair = mintKeypair;
    global.vestingAccount = vestingAccount;
  });

  it("Creates a repository with imported spl token", async () => {
    //generates key pairs and airdrops solana to them
    const mintAuthority = await create_keypair();
    let [
      nameRouterAccount,
      routerCreatorKeypair,
      repositoryCreator,
      repositoryCreatorVerifiedAccount,
    ] = [
      global.nameRouterAccount,
      global.routerCreatorKeypair,
      global.repositoryCreator,
      global.repositoryCreatorVerifiedAccount,
    ];
    let secondId = constant.repositoryId + "2";
    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
    ] = await create_spl_token(repositoryCreator, secondId);

    const mintAddress = await createMint(
      connection,
      mintAuthority,
      mintAuthority.publicKey,
      mintAuthority.publicKey,
      6
    );

    await program.methods
      .createRepository(
        secondId,
        constant.repositoryTitle,
        constant.repositoryUri,
        null,
        null,
        null
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount: null,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: repositoryCreatorVerifiedAccount,
        rewardsMint: null,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: null,
        vestingTokenAccount: null,
        tokenMetadataProgram: constant.TOKEN_METADATA_PROGRAM_ID,
        metadata: null,
        importedMint: mintAddress,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });

  it("Creates a issue", async () => {
    let [
      nameRouterAccount,
      routerCreatorKeypair,
      repositoryCreator,
      repositoryAccount,
    ] = [
      global.nameRouterAccount,
      global.routerCreatorKeypair,
      global.repositoryCreator,
      global.repositoryAccount,
    ];

    const issueCreatorKeypair = await create_keypair();

    const { issueIndex } = await program.account.repository.fetch(
      repositoryAccount
    );

    // Adding issue creator user

    const [issueVerifiedUser] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      issueCreatorKeypair.publicKey
    );
    // Creating issue
    const issueURI = `https://github.com/${constant.userName}/${constant.repositoryId}/issues/${issueIndex}`;

    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    await program.methods
      .addIssue(issueURI)
      .accounts({
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        issueVerifiedUser,
        nameRouterAccount,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: repositoryCreator.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([issueCreatorKeypair])
      .rpc(rpcConfig);
    global.issueCreator = issueCreatorKeypair;
    global.issueVerifiedUser = issueVerifiedUser;
    global.issueAccount = issueAccount;
  });

  it("Stakes on a issue", async () => {
    let [
      repositoryCreator,
      repositoryAccount,
      mintKeypair,
      issueAccount,
      vestingAccount,
    ] = [
      global.repositoryCreator,
      global.repositoryAccount,
      global.mintKeypair,
      global.issueAccount,
      global.vestingAccount,
    ];
    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
      issueAccount,
      true
    );

    const repositoryCreatorTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      repositoryCreator.publicKey
    );

    const vestingTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      vestingAccount,
      true
    );
    // Staking tokens on a issue
    const [issueStakerAccount] = await get_pda_from_seeds([
      Buffer.from("issuestaker"),
      issueAccount.toBuffer(),
      repositoryCreator.publicKey.toBuffer(),
    ]);

    await program.methods
      .unlockTokens()
      .accounts({
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        tokenMint: mintKeypair,
        vestingTokenAccount: vestingTokenAccount,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

    await program.methods
      .stakeIssue(new anchor.BN(10))
      .accounts({
        issueAccount,
        repositoryAccount,
        issueTokenPoolAccount,
        issueStaker: repositoryCreator.publicKey,
        issueStakerAccount,
        issueStakerTokenAccount: repositoryCreatorTokenAccount,
        rewardsMint: mintKeypair,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        pullRequestMetadataAccount: null,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });

  it("Unstakes on a issue", async () => {
    let [repositoryCreator, repositoryAccount, mintKeypair, issueAccount] = [
      global.repositoryCreator,
      global.repositoryAccount,
      global.mintKeypair,
      global.issueAccount,
    ];

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
      issueAccount,
      true
    );
    const repositoryCreatorTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      repositoryCreator.publicKey
    );

    const [issueStakerAccount] = await get_pda_from_seeds([
      Buffer.from("issuestaker"),
      issueAccount.toBuffer(),
      repositoryCreator.publicKey.toBuffer(),
    ]);

    await program.methods
      .unstakeIssue()
      .accounts({
        issueAccount,
        repositoryAccount,
        issueTokenPoolAccount,
        issueStaker: repositoryCreator.publicKey,
        issueStakerAccount,
        issueStakerTokenAccount: repositoryCreatorTokenAccount,
        rewardsMint: mintKeypair,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        pullRequestMetadataAccount: null,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

    //want to have some stake on issue for future tests
    await program.methods
      .stakeIssue(new anchor.BN(10))
      .accounts({
        issueAccount,
        repositoryAccount,
        issueTokenPoolAccount,
        issueStaker: repositoryCreator.publicKey,
        issueStakerAccount,
        issueStakerTokenAccount: repositoryCreatorTokenAccount,
        rewardsMint: mintKeypair,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        pullRequestMetadataAccount: null,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });

  it("Creates a roadmap!", async () => {
    let [
      repositoryCreator,
      repositoryAccount,
      nameRouterAccount,
      repositoryCreatorVerifiedAccount,
      routerCreatorKeypair,
    ] = [
      global.repositoryCreator,
      global.repositoryAccount,
      global.nameRouterAccount,
      global.repositoryCreatorVerifiedAccount,
      global.routerCreatorKeypair,
    ];

    const [metadataAccount] = await get_pda_from_seeds([
      Buffer.from("roadmapmetadataadd"),
      repositoryAccount.toBuffer(),
      repositoryCreator.publicKey.toBuffer(),
    ]);

    await program.methods
      .addRoadmapData(
        constant.roadmapTitle,
        constant.roadmapDescription,
        constant.roadmapImageUrl,
        constant.roadmapOutlook
      )
      .accounts({
        nameRouterAccount,
        metadataAccount,
        roadmapDataAdder: repositoryCreator.publicKey,
        roadmapVerifiedUser: repositoryCreatorVerifiedAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryAccount: repositoryAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
    global.roadmapMetadataAccount = metadataAccount;
  });
  it("Add an objective to a roadmap!", async () => {
    let [
      repositoryCreator,
      repositoryAccount,
      nameRouterAccount,
      routerCreatorKeypair,
      repositoryCreatorVerifiedAccount,
      roadmapMetadataAccount,
    ] = [
      global.repositoryCreator,
      global.repositoryAccount,
      global.nameRouterAccount,
      global.routerCreatorKeypair,
      global.repositoryCreatorVerifiedAccount,
      global.roadmapMetadataAccount,
    ];

    const [objectiveAccount] = await get_pda_from_seeds([
      Buffer.from("objectivedataadd"),
      repositoryCreator.publicKey.toBuffer(),
      Buffer.from(constant.objectiveId),
    ]);

    await program.methods
      .addObjectiveData(
        constant.objectiveId,
        constant.objectiveTitle,
        constant.objectiveStartUnix,
        constant.objectiveDescription,
        constant.objectiveDeliverable
      )
      .accounts({
        nameRouterAccount,
        metadataAccount: objectiveAccount,
        roadmapMetadataAccount: roadmapMetadataAccount,
        parentObjectiveAccount: null,
        objectiveDataAddr: repositoryCreator.publicKey,
        objectiveVerifiedUser: repositoryCreatorVerifiedAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryAccount: repositoryAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
    global.rootObjectiveAccount = objectiveAccount;
  });
  it("Add a child objective to an objective", async () => {
    let [
      repositoryCreator,
      repositoryAccount,
      nameRouterAccount,
      routerCreatorKeypair,
      repositoryCreatorVerifiedAccount,
      objectiveAccount,
    ] = [
      global.repositoryCreator,
      global.repositoryAccount,
      global.nameRouterAccount,
      global.routerCreatorKeypair,
      global.repositoryCreatorVerifiedAccount,
      global.rootObjectiveAccount,
    ];

    const [objectiveAccount2] = await get_pda_from_seeds([
      Buffer.from("objectivedataadd"),
      repositoryCreator.publicKey.toBuffer(),
      Buffer.from("2"),
    ]);
    await program.methods
      .addObjectiveData(
        "2",
        constant.objectiveTitle,
        constant.objectiveStartUnix,
        constant.objectiveDescription,
        constant.objectiveDeliverable
      )
      .accounts({
        nameRouterAccount,
        metadataAccount: objectiveAccount2,
        objectiveDataAddr: repositoryCreator.publicKey,
        roadmapMetadataAccount: null,
        parentObjectiveAccount: objectiveAccount,
        objectiveVerifiedUser: repositoryCreatorVerifiedAccount,
        repositoryAccount: repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });
  it("Adds a PR to an issue", async () => {
    let [
      repositoryAccount,
      mintKeypair,
      issueAccount,
      routerCreatorKeypair,
      nameRouterAccount,
    ] = [
      global.repositoryAccount,
      global.mintKeypair,
      global.issueAccount,
      global.routerCreatorKeypair,
      global.nameRouterAccount,
    ];

    const pullRequestCreator = await create_keypair();
    const [pullRequestCreatorVerifiedAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      pullRequestCreator.publicKey
    );

    const [pullRequestMetadataAccount] = await get_pda_from_seeds([
      Buffer.from("pullrequestadded"),
      issueAccount.toBuffer(),
      pullRequestCreator.publicKey.toBuffer(),
    ]);

    const pullRequestTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      pullRequestMetadataAccount,
      true
    );

    await program.methods
      .addPr(constant.pullRequestMetadataUri)
      .accounts({
        pullRequestVerifiedUser: pullRequestCreatorVerifiedAccount,
        issue: issueAccount,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        nameRouterAccount,
        pullRequestTokenAccount,
        pullRequestAddr: pullRequestCreator.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rewardsMint: mintKeypair,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([pullRequestCreator])
      .rpc(rpcConfig);
    global.pullRequestCreator = pullRequestCreator;
    global.pullRequestCreatorVerifiedAccount =
      pullRequestCreatorVerifiedAccount;
    global.pullRequestMetadataAccount = pullRequestMetadataAccount;
  });
  it("Accepts a PR", async () => {
    let [
      repositoryCreator,
      repositoryAccount,
      pullRequestMetadataAccount,
      pullRequestCreator,
      issueAccount,
    ] = [
      global.repositoryCreator,
      global.repositoryAccount,
      global.pullRequestMetadataAccount,
      global.pullRequestCreator,
      global.issueAccount,
    ];

    await program.methods
      .acceptPr(constant.repositoryId)
      .accounts({
        pullRequestAddr: pullRequestCreator.publicKey,
        pullRequestMetadataAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryAccount,
        issue: issueAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });

  it("Getting a PR accepted and getting rewarded for it", async () => {
    let [
      repositoryCreator,
      repositoryAccount,
      pullRequestMetadataAccount,
      pullRequestCreator,
      issueAccount,
      mintKeypair,
      issueCreator,
    ] = [
      global.repositoryCreator,
      global.repositoryAccount,
      global.pullRequestMetadataAccount,
      global.pullRequestCreator,
      global.issueAccount,
      global.mintKeypair,
      global.issueCreator,
    ];

    const pullRequestCreatorRewardAccount = await getAssociatedTokenAddress(
      mintKeypair,
      pullRequestCreator.publicKey
    );

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
      issueAccount,
      true
    );

    await program.methods
      .claimReward()
      .accounts({
        pullRequestCreator: pullRequestCreator.publicKey,
        pullRequest: pullRequestMetadataAccount,
        pullRequestCreatorRewardAccount,
        repositoryCreator: repositoryCreator.publicKey,
        rewardsMint: mintKeypair,
        repositoryAccount,
        issueAccount: issueAccount,
        issueTokenPoolAccount,
        issueCreator: issueCreator.publicKey,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([pullRequestCreator])
      .rpc(rpcConfig);
  });

  it("Create communal account to store tokens", async () => {
    let [repositoryCreator, mintKeypair] = [
      global.repositoryCreator,
      global.mintKeypair,
    ];

    const [communal_account] = await get_pda_from_seeds([
      Buffer.from("are_we_conscious"),
      Buffer.from("is love life ?  "),
      Buffer.from("arewemadorinlove"),
      mintKeypair.toBuffer(),
    ]);

    const communalTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      communal_account,
      true
    );

    await program.methods
      .createCommunalAccount()
      .accounts({
        authority: repositoryCreator.publicKey,
        communalDeposit: communal_account,
        communalTokenAccount: communalTokenAccount,
        systemProgram: web3.SystemProgram.programId,
        rewardsMint: mintKeypair,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        usdcMint: mintKeypair,
        communalUsdcAccount: communalTokenAccount,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
    global.communalAccount = communal_account;
  });

  it("Sends a buy transaction", async () => {
    let [repositoryCreator, mintKeypair, communalAccount, repositoryAccount] = [
      global.repositoryCreator,
      global.mintKeypair,
      global.communalAccount,
      global.repositoryAccount,
    ];

    const communalTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      communalAccount,
      true
    );

    const repositoryCreatorTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      repositoryCreator.publicKey
    );

    await program.methods
      .buyTokens(new anchor.BN(1), new anchor.BN(1))
      .accounts({
        buyer: repositoryCreator.publicKey,
        communalDeposit: communalAccount,
        communalTokenAccount: communalTokenAccount,
        rewardsMint: mintKeypair,
        tokenProgram: TOKEN_PROGRAM_ID,
        repositoryAccount: repositoryAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
        buyerTokenAccount: repositoryCreatorTokenAccount,
        communalUsdcAccount: communalTokenAccount,
        buyerUsdcAccount: repositoryCreatorTokenAccount,
        usdcMint: mintKeypair,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });

  it("Sends a sell transaction", async () => {
    let [repositoryCreator, mintKeypair, communalAccount, repositoryAccount] = [
      global.repositoryCreator,
      global.mintKeypair,
      global.communalAccount,
      global.repositoryAccount,
    ];

    const communalTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      communalAccount,
      true
    );

    const repositoryCreatorTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      repositoryCreator.publicKey
    );

    await program.methods
      .sellTokens(new anchor.BN(0), new anchor.BN(1))
      .accounts({
        seller: repositoryCreator.publicKey,
        communalDeposit: communalAccount,
        communalTokenAccount: communalTokenAccount,
        rewardsMint: mintKeypair,
        repositoryAccount: repositoryAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
        sellerTokenAccount: repositoryCreatorTokenAccount,
        communalUsdcAccount: communalTokenAccount,
        sellerUsdcAccount: repositoryCreatorTokenAccount,
        usdcMint: mintKeypair,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });

  it("Grant money to objective", async () => {
    let [mintKeypair, repositoryAccount, objectiveAccount, repositoryCreator] =
      [
        global.mintKeypair,
        global.repositoryAccount,
        global.rootObjectiveAccount,
        global.repositoryCreator,
      ];

    const grantee = await create_keypair();
    const [verifiedUserAccount] = await create_verified_user(
      global.routerCreatorKeypair,
      global.nameRouterAccount,
      grantee.publicKey
    );

    const [granteeAccount] = await get_pda_from_seeds([
      grantee.publicKey.toBuffer(),
      repositoryAccount.toBuffer(),
      objectiveAccount.toBuffer(),
    ]);

    const objectiveStakeAccount = await getAssociatedTokenAddress(
      mintKeypair,
      objectiveAccount,
      true
    );

    const granteeStakeAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      grantee,
      mintKeypair,
      grantee.publicKey,
      false
    );

    const repositoryCreatorTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      repositoryCreator.publicKey
    );

    await transfer(
      connection,
      repositoryCreator,
      repositoryCreatorTokenAccount,
      granteeStakeAccount.address,
      repositoryCreator,
      10
    );

    await program.methods
      .grantMoney(new anchor.BN(10), constant.roadmapImageUrl)
      .accounts({
        grantee: grantee.publicKey,
        granteeVerifiedUser: verifiedUserAccount,
        objective: objectiveAccount,
        repository: repositoryAccount,
        tokenMint: mintKeypair,
        granteeAccount: granteeAccount,
        objectiveStakeAccount: objectiveStakeAccount,
        granteeStakeAccount: granteeStakeAccount.address,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([grantee])
      .rpc(rpcConfig);
    global.grantee = grantee;
    global.granteeAccount = granteeAccount;
  });
  it("Disperse grant money", async () => {
    let [
      repositoryAccount,
      objectiveAccount,
      repositoryCreator,
      issueAccount,
      mintKeypair,
    ] = [
      global.repositoryAccount,
      global.rootObjectiveAccount,
      global.repositoryCreator,
      global.issueAccount,
      global.mintKeypair,
    ];

    const objectiveStakeAccount = await getAssociatedTokenAddress(
      mintKeypair,
      objectiveAccount,
      true
    );
    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
      issueAccount,
      true
    );
    await program.methods
      .disperseGrant(new anchor.BN(10))
      .accounts({
        repositoryCreator: repositoryCreator.publicKey,
        objective: objectiveAccount,
        objectiveStakeAccount: objectiveStakeAccount,
        repository: repositoryAccount,
        issueAccount: issueAccount,
        issueTokenPoolAccount: issueTokenPoolAccount,
        systemProgram: web3.SystemProgram.programId,
        tokenMint: mintKeypair,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });
});
