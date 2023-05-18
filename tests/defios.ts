import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Defios } from "../target/types/defios";

import * as ed from "@noble/ed25519";
import { PublicKey } from "@saberhq/solana-contrib";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  createMintToCheckedInstruction,
  getAssociatedTokenAddress,
  MintLayout,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

import sha256 from "sha256";
import { Keypair, ComputeBudgetProgram } from "@solana/web3.js";
import { min } from "bn.js";

describe("defios", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  //testing defios workspace here
  const program = anchor.workspace.Defios as Program<Defios>;
  const {
    provider: { connection },
  } = program;

  const { web3 } = anchor;

  //global variables for tests
  const signatureVersion = 1;
  const signingName = "defios.com";
  const userName: string = "sunguru98";
  const userPubkey = new PublicKey(
    "81sWMLg1EgYps3nMwyeSW1JfjKgFqkGYPP85vTnkFzRn"
  );
  const repositoryName = "defios";
  const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
    units: 1000000,
  });

  const addPriorityFee = ComputeBudgetProgram.setComputeUnitPrice({
    microLamports: 1,
  });
  const roadmapTitle = "Test Roadmap";
  const roadmapDescription = "https://github.com/defi-os/Issues";
  const roadmapOutlook = 0;
  const objectiveDeliverable = 0;
  const objectiveTitle = "Test Objective";
  const objectiveDescription = "https://github.com/defi-os/Issues";
  const objectiveEndUnix = new anchor.BN(1735603200);
  const objectiveStartUnix = new anchor.BN(1704067200);
  const pull_request_metadata_uri = "https://github.com/defi-os/Issues";
  //helper functions
  async function create_keypair() {
    const keypair = web3.Keypair.generate();
    await connection.confirmTransaction(
      {
        signature: await connection.requestAirdrop(
          keypair.publicKey,
          web3.LAMPORTS_PER_SOL
        ),
        ...(await connection.getLatestBlockhash()),
      },
      "confirmed"
    );
    return keypair;
  }

  async function get_pda_from_seeds(seeds) {
    return await web3.PublicKey.findProgramAddressSync(
      seeds,
      program.programId
    );
  }

  //main testsuite code
  async function create_name_router() {
    //generating keypair and airdropping solana to it
    const routerCreatorKeypair = await create_keypair();

    //get public key of pda ideally generated using seeds
    const [nameRouterAccount] = await get_pda_from_seeds([
      Buffer.from(signingName),
      Buffer.from(signatureVersion.toString()),
      routerCreatorKeypair.publicKey.toBuffer(),
    ]);

    //call create name router function
    await program.methods
      .createNameRouter(signingName, signatureVersion)
      .accounts({
        nameRouterAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([routerCreatorKeypair])
      .rpc({ commitment: "confirmed" });
    return [routerCreatorKeypair, nameRouterAccount];
  }

  async function create_verified_user(
    routerCreatorKeypair,
    nameRouterAccount,
    pubKey
  ) {
    // Signature test
    //Create byte array of message
    const message = Uint8Array.from(
      Buffer.from(`DefiOS(${userName}, ${userPubkey.toString()})`)
    );

    //create signature from message and secret key
    const signature = await ed.sign(
      message,
      routerCreatorKeypair.secretKey.slice(0, 32)
    );

    //create instruction from message, public key, and signature of account
    const createED25519Ix = web3.Ed25519Program.createInstructionWithPublicKey({
      message: message,
      publicKey: routerCreatorKeypair.publicKey.toBytes(),
      signature,
    });

    //gets public key from seeds
    const [verifiedUserAccount] = await get_pda_from_seeds([
      Buffer.from(userName),
      pubKey.toBuffer(),
      nameRouterAccount.toBuffer(),
    ]);

    //calls add verified user method
    await program.methods
      .addVerifiedUser(
        userName,
        pubKey,
        Buffer.from(message),
        Buffer.from(signature)
      )
      .accounts({
        nameRouterAccount,
        verifiedUserAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        sysvarInstructions: web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([routerCreatorKeypair])
      .preInstructions([createED25519Ix])
      .rpc({ commitment: "confirmed" });
    return [verifiedUserAccount];
  }

  async function create_spl_token(repositoryCreator) {
    // Creating rewards mint
    const mintKeypair = web3.Keypair.generate();
    const createAccountIx = web3.SystemProgram.createAccount({
      programId: TOKEN_PROGRAM_ID,
      fromPubkey: repositoryCreator.publicKey,
      newAccountPubkey: mintKeypair.publicKey,
      lamports: await connection.getMinimumBalanceForRentExemption(
        MintLayout.span
      ),
      space: MintLayout.span,
    });

    const initMintIx = createInitializeMintInstruction(
      mintKeypair.publicKey,
      9,
      repositoryCreator.publicKey,
      repositoryCreator.publicKey
    );

    const repositoryCreatorRewardsTokenAccount =
      await getAssociatedTokenAddress(
        mintKeypair.publicKey,
        repositoryCreator.publicKey
      );

    // Creating repository
    const [repositoryAccount] = await get_pda_from_seeds([
      Buffer.from("repository"),
      Buffer.from(repositoryName),
      repositoryCreator.publicKey.toBuffer(),
    ]);

    const [vestingAccount] = await get_pda_from_seeds([
      Buffer.from("vesting"),
      mintKeypair.publicKey.toBuffer(),
      repositoryAccount.toBuffer(),
    ]);

    const vestingTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      vestingAccount,
      true
    );

    const createAssociatedTokenIx = createAssociatedTokenAccountInstruction(
      repositoryCreator.publicKey,
      vestingTokenAccount,
      vestingAccount,
      mintKeypair.publicKey
    );

    const mintTokensIx = createMintToCheckedInstruction(
      mintKeypair.publicKey,
      vestingTokenAccount,
      repositoryCreator.publicKey,
      10 ** 4,
      9,
      []
    );

    const preInstructions = [
      createAccountIx,
      initMintIx,
      createAssociatedTokenIx,
      mintTokensIx,
    ];

    const repositoryCreatorTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      repositoryCreator.publicKey
    );

    return [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      preInstructions,
    ];
  }
  //main testsuite
  //creating a name router
  it("Creates a name router!", async () => {
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    //get data related to name router pda
    const {
      routerCreator,
      signatureVersion: fSignatureVersion,
      signingDomain,
      bump,
      totalVerifiedUsers,
    } = await program.account.nameRouter.fetch(nameRouterAccount);
  });

  it("Adds a verified user", async () => {
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();

    const [verifiedUserAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      userPubkey
    );
  });

  it("Creates a repository", async () => {
    //generates key pairs and airdrops solana to them
    const repositoryCreator = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [verifiedUserAccount] = await create_verified_user(
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
      preInstructions,
    ] = await create_spl_token(repositoryCreator);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([repositoryCreator, mintKeypair])
      .rpc({ skipPreflight: false });
  });

  it("Creates a issue", async () => {
    const repositoryCreator = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [repositoryVerifiedUser] = await create_verified_user(
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
      preInstructions,
    ] = await create_spl_token(repositoryCreator);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: repositoryVerifiedUser,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([repositoryCreator, mintKeypair])
      .rpc();

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
    const issueURI = `https://github.com/${userName}/${repositoryName}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueAccount,
      true
    );

    await program.methods
      .addIssue(issueURI)
      .accounts({
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        issueTokenPoolAccount,
        issueVerifiedUser,
        nameRouterAccount,
        repositoryAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: repositoryCreator.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc();
  });

  it("Stakes on a issue", async () => {
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const repositoryCreator = await create_keypair();
    const issueCreatorKeypair = await create_keypair();
    const issueStakerKeypair = await create_keypair();
    const [repositoryVerifiedUser] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      repositoryCreator.publicKey
    );

    // Creating repository
    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      preInstructions,
    ] = await create_spl_token(repositoryCreator);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: repositoryVerifiedUser,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([repositoryCreator, mintKeypair])
      .rpc();

    const { issueIndex } = await program.account.repository.fetch(
      repositoryAccount
    );

    // Adding issue creator user
    const issueCreatorUserName: string = "abhibasu";
    const issueCreatorMessage = Uint8Array.from(
      Buffer.from(
        `DefiOS(${issueCreatorUserName}, ${issueCreatorKeypair.publicKey.toString()})`
      )
    );

    const [issueVerifiedUser] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      issueCreatorKeypair.publicKey
    );

    // Creating issue
    const issueURI = `https://github.com/${userName}/${repositoryName}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueAccount,
      true
    );

    await program.methods
      .addIssue(issueURI)
      .accounts({
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        issueTokenPoolAccount,
        issueVerifiedUser,
        nameRouterAccount,
        repositoryAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: repositoryCreator.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc();

    await program.account.issue.fetch(issueAccount);

    // Staking tokens on a issue
    const transferAmount = 1000 * 10 ** 9;
    const issueStakerTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueStakerKeypair.publicKey
    );

    const createIssueStakerTokenAccountIx =
      createAssociatedTokenAccountInstruction(
        issueStakerKeypair.publicKey,
        issueStakerTokenAccount,
        issueStakerKeypair.publicKey,
        mintKeypair.publicKey
      );

    const mintToIssueStakerIx = createMintToCheckedInstruction(
      mintKeypair.publicKey,
      issueStakerTokenAccount,
      repositoryCreator.publicKey,
      transferAmount,
      9,
      []
    );

    const [issueStakerAccount] = await get_pda_from_seeds([
      Buffer.from("issuestaker"),
      issueAccount.toBuffer(),
      issueStakerKeypair.publicKey.toBuffer(),
    ]);

    await program.methods
      .stakeIssue(new anchor.BN(transferAmount))
      .accounts({
        issueAccount,
        repositoryAccount,
        issueTokenPoolAccount,
        issueStaker: issueStakerKeypair.publicKey,
        issueStakerAccount,
        issueStakerTokenAccount,
        rewardsMint: mintKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .preInstructions([createIssueStakerTokenAccountIx, mintToIssueStakerIx])
      .signers([repositoryCreator, issueStakerKeypair])
      .rpc();
  });

  it("Unstakes on a issue", async () => {
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const repositoryCreator = await create_keypair();
    const issueCreatorKeypair = await create_keypair();
    const issueStakerKeypair = await create_keypair();

    // Adding repository creator user
    const [repositoryVerifiedUser] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      repositoryCreator.publicKey
    );

    // Creating rewards mint
    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      preInstructions,
    ] = await create_spl_token(repositoryCreator);

    // Creating repository
    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: repositoryVerifiedUser,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([repositoryCreator, mintKeypair])
      .rpc();

    const { issueIndex } = await program.account.repository.fetch(
      repositoryAccount
    );

    // Adding issue creator user
    const issueCreatorUserName: string = "abhibasu";
    const issueCreatorMessage = Uint8Array.from(
      Buffer.from(
        `DefiOS(${issueCreatorUserName}, ${issueCreatorKeypair.publicKey.toString()})`
      )
    );

    const issueCreatorSignature = await ed.sign(
      issueCreatorMessage,
      routerCreatorKeypair.secretKey.slice(0, 32)
    );

    const createED25519IxIssueCreator =
      web3.Ed25519Program.createInstructionWithPublicKey({
        message: issueCreatorMessage,
        publicKey: routerCreatorKeypair.publicKey.toBytes(),
        signature: issueCreatorSignature,
      });

    const [issueVerifiedUser] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      issueCreatorKeypair.publicKey
    );

    // Creating issue
    const issueURI = `https://github.com/${userName}/${repositoryName}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueAccount,
      true
    );

    await program.methods
      .addIssue(issueURI)
      .accounts({
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        issueTokenPoolAccount,
        issueVerifiedUser,
        nameRouterAccount,
        repositoryAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: repositoryCreator.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc({ skipPreflight: true });

    await program.account.issue.fetch(issueAccount);

    // Staking tokens on a issue
    const transferAmount = 1000 * 10 ** 9;
    const issueStakerTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueStakerKeypair.publicKey
    );

    const createIssueStakerTokenAccountIx =
      createAssociatedTokenAccountInstruction(
        issueStakerKeypair.publicKey,
        issueStakerTokenAccount,
        issueStakerKeypair.publicKey,
        mintKeypair.publicKey
      );

    const mintToIssueStakerIx = createMintToCheckedInstruction(
      mintKeypair.publicKey,
      issueStakerTokenAccount,
      repositoryCreator.publicKey,
      transferAmount,
      9,
      []
    );

    const [issueStakerAccount] = await get_pda_from_seeds([
      Buffer.from("issuestaker"),
      issueAccount.toBuffer(),
      issueStakerKeypair.publicKey.toBuffer(),
    ]);

    await program.methods
      .stakeIssue(new anchor.BN(transferAmount))
      .accounts({
        issueAccount,
        repositoryAccount,
        issueTokenPoolAccount,
        issueStaker: issueStakerKeypair.publicKey,
        issueStakerAccount,
        issueStakerTokenAccount,
        rewardsMint: mintKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .preInstructions([createIssueStakerTokenAccountIx, mintToIssueStakerIx])
      .signers([repositoryCreator, issueStakerKeypair])
      .rpc();

    await program.methods
      .unstakeIssue()
      .accounts({
        issueAccount,
        repositoryAccount,
        issueTokenPoolAccount,
        issueStaker: issueStakerKeypair.publicKey,
        issueStakerAccount,
        issueStakerTokenAccount,
        rewardsMint: mintKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueStakerKeypair])
      .rpc({ skipPreflight: true });
  });

  it("Adds a commit to an issue", async () => {
    const repositoryCreator = await create_keypair();
    const issueCreatorKeypair = await create_keypair();
    const issueStakerKeypair = await create_keypair();
    const commitCreatorKeypair = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [repositoryVerifiedUser] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      repositoryCreator.publicKey
    );

    // Creating rewards mint
    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      preInstructions,
    ] = await create_spl_token(repositoryCreator);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: repositoryVerifiedUser,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([repositoryCreator, mintKeypair])
      .rpc();

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
    const issueURI = `https://github.com/${userName}/${repositoryName}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueAccount,
      true
    );

    await program.methods
      .addIssue(issueURI)
      .accounts({
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        issueTokenPoolAccount,
        issueVerifiedUser,
        nameRouterAccount,
        repositoryAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: repositoryCreator.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc({ skipPreflight: true });

    await program.account.issue.fetch(issueAccount);

    // Staking tokens on a issue
    const transferAmount = 1000 * 10 ** 9;
    const issueStakerTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueStakerKeypair.publicKey
    );

    const createIssueStakerTokenAccountIx =
      createAssociatedTokenAccountInstruction(
        issueStakerKeypair.publicKey,
        issueStakerTokenAccount,
        issueStakerKeypair.publicKey,
        mintKeypair.publicKey
      );

    const mintToIssueStakerIx = createMintToCheckedInstruction(
      mintKeypair.publicKey,
      issueStakerTokenAccount,
      repositoryCreator.publicKey,
      transferAmount,
      9,
      []
    );

    const [issueStakerAccount] = await get_pda_from_seeds([
      Buffer.from("issuestaker"),
      issueAccount.toBuffer(),
      issueStakerKeypair.publicKey.toBuffer(),
    ]);

    await program.methods
      .stakeIssue(new anchor.BN(transferAmount))
      .accounts({
        issueAccount,
        repositoryAccount,
        issueTokenPoolAccount,
        issueStaker: issueStakerKeypair.publicKey,
        issueStakerAccount,
        issueStakerTokenAccount,
        rewardsMint: mintKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .preInstructions([createIssueStakerTokenAccountIx, mintToIssueStakerIx])
      .signers([repositoryCreator, issueStakerKeypair])
      .rpc();

    // Adding commit creator user
    const [commitVerifiedUser] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      commitCreatorKeypair.publicKey
    );

    // Adding a commit
    const treeHash = sha256("Tree hash 1").slice(0, 8);
    const commitHash = sha256("Commit hash 1").slice(0, 8);
    const metadataURI =
      "https://arweave.net/jB7pLq6IReTCeJRHhXiYrfhdEFBeZEDppMc8fkxvJj0";

    const [commitAccount] = await get_pda_from_seeds([
      Buffer.from("commit"),
      Buffer.from(commitHash),
      commitCreatorKeypair.publicKey.toBuffer(),
      issueAccount.toBuffer(),
    ]);

    await program.methods
      .addCommit(commitHash, treeHash, metadataURI)
      .accounts({
        commitAccount,
        commitCreator: commitCreatorKeypair.publicKey,
        commitVerifiedUser,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        nameRouterAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([commitCreatorKeypair])
      .rpc({ skipPreflight: true });
  });

  it("Creates a roadmap!", async () => {
    //generates key pairs and airdrops solana to them
    const roadmapDataAdder = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [verifiedUserAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      roadmapDataAdder.publicKey
    );

    //adds logs to keypair

    const [metadataAccount] = await get_pda_from_seeds([
      Buffer.from("roadmapmetadataadd"),
      verifiedUserAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);
    await program.methods
      .addRoadmapData(roadmapTitle, roadmapDescription, roadmapOutlook)
      .accounts({
        nameRouterAccount,
        metadataAccount,
        roadmapDataAdder: roadmapDataAdder.publicKey,
        roadmapVerifiedUser: verifiedUserAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });
  });
  it("Creates an objective!", async () => {
    //generates key pairs and airdrops solana to them
    const roadmapDataAdder = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [verifiedUserAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      roadmapDataAdder.publicKey
    );

    //adds logs to keypair

    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      preInstructions,
    ] = await create_spl_token(roadmapDataAdder);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([roadmapDataAdder, mintKeypair])
      .rpc();

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
    const issueURI = `https://github.com/${userName}/${repositoryName}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueAccount,
      true
    );

    await program.methods
      .addIssue(issueURI)
      .accounts({
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        issueTokenPoolAccount,
        issueVerifiedUser,
        nameRouterAccount,
        repositoryAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc();

    const [metadataAccount] = await get_pda_from_seeds([
      Buffer.from("roadmapmetadataadd"),
      verifiedUserAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);
    await program.methods
      .addRoadmapData(roadmapTitle, roadmapDescription, roadmapOutlook)
      .accounts({
        nameRouterAccount,
        metadataAccount,
        roadmapDataAdder: roadmapDataAdder.publicKey,
        roadmapVerifiedUser: verifiedUserAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });
    let objective_number = 1;
    const objectiveId: string = objective_number.toString();
    const [objectiveAccount] = await get_pda_from_seeds([
      Buffer.from("objectivedataadd"),
      issueAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
      Buffer.from(objectiveId),
    ]);
    await program.methods
      .addObjectiveData(
        objectiveId,
        objectiveTitle,
        objectiveStartUnix,
        objectiveEndUnix,
        objectiveDescription,
        objectiveDeliverable
      )
      .accounts({
        nameRouterAccount,
        objectiveIssue: issueAccount,
        metadataAccount: objectiveAccount,
        objectiveDataAddr: roadmapDataAdder.publicKey,
        objectiveVerifiedUser: verifiedUserAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });
  });
  it("Add an objective to a roadmap!", async () => {
    //generates key pairs and airdrops solana to them
    const roadmapDataAdder = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [verifiedUserAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      roadmapDataAdder.publicKey
    );

    //adds logs to keypair

    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      preInstructions,
    ] = await create_spl_token(roadmapDataAdder);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([roadmapDataAdder, mintKeypair])
      .rpc();

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
    const issueURI = `https://github.com/${userName}/${repositoryName}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueAccount,
      true
    );

    await program.methods
      .addIssue(issueURI)
      .accounts({
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        issueTokenPoolAccount,
        issueVerifiedUser,
        nameRouterAccount,
        repositoryAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc();

    const [metadataAccount] = await get_pda_from_seeds([
      Buffer.from("roadmapmetadataadd"),
      verifiedUserAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);
    await program.methods
      .addRoadmapData(roadmapTitle, roadmapDescription, roadmapOutlook)
      .accounts({
        nameRouterAccount,
        metadataAccount,
        roadmapDataAdder: roadmapDataAdder.publicKey,
        roadmapVerifiedUser: verifiedUserAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    const [objectiveAccount] = await get_pda_from_seeds([
      Buffer.from("objectivedataadd"),
      issueAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
      Buffer.from("1"),
    ]);
    await program.methods
      .addObjectiveData(
        "1",
        objectiveTitle,
        objectiveStartUnix,
        objectiveEndUnix,
        objectiveDescription,
        objectiveDeliverable
      )
      .accounts({
        nameRouterAccount,
        objectiveIssue: issueAccount,
        metadataAccount: objectiveAccount,
        objectiveDataAddr: roadmapDataAdder.publicKey,
        objectiveVerifiedUser: verifiedUserAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    await program.methods
      .addChildObjective()
      .accounts({
        nameRouterAccount,
        objectiveAccount: objectiveAccount,
        roadmapMetadataAccount: metadataAccount,
        childObjectiveAdder: roadmapDataAdder.publicKey,
        objectiveVerifiedUser: verifiedUserAccount,
        parentAccount: null,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });
  });
  it("Add a child objective to an objective", async () => {
    //generates key pairs and airdrops solana to them
    const roadmapDataAdder = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [verifiedUserAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      roadmapDataAdder.publicKey
    );

    //adds logs to keypair

    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      preInstructions,
    ] = await create_spl_token(roadmapDataAdder);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([roadmapDataAdder, mintKeypair])
      .rpc();

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
    const issueURI = `https://github.com/${userName}/${repositoryName}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueAccount,
      true
    );

    await program.methods
      .addIssue(issueURI)
      .accounts({
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        issueTokenPoolAccount,
        issueVerifiedUser,
        nameRouterAccount,
        repositoryAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc();

    const [metadataAccount] = await get_pda_from_seeds([
      Buffer.from("roadmapmetadataadd"),
      verifiedUserAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);
    await program.methods
      .addRoadmapData(roadmapTitle, roadmapDescription, roadmapOutlook)
      .accounts({
        nameRouterAccount,
        metadataAccount,
        roadmapDataAdder: roadmapDataAdder.publicKey,
        roadmapVerifiedUser: verifiedUserAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    const [objectiveAccount] = await get_pda_from_seeds([
      Buffer.from("objectivedataadd"),
      issueAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
      Buffer.from("1"),
    ]);
    await program.methods
      .addObjectiveData(
        "1",
        objectiveTitle,
        objectiveStartUnix,
        objectiveEndUnix,
        objectiveDescription,
        objectiveDeliverable
      )
      .accounts({
        nameRouterAccount,
        objectiveIssue: issueAccount,
        metadataAccount: objectiveAccount,
        objectiveDataAddr: roadmapDataAdder.publicKey,
        objectiveVerifiedUser: verifiedUserAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    const [objectiveAccount2] = await get_pda_from_seeds([
      Buffer.from("objectivedataadd"),
      issueAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
      Buffer.from("2"),
    ]);
    await program.methods
      .addObjectiveData(
        "2",
        objectiveTitle,
        objectiveStartUnix,
        objectiveEndUnix,
        objectiveDescription,
        objectiveDeliverable
      )
      .accounts({
        nameRouterAccount,
        objectiveIssue: issueAccount,
        metadataAccount: objectiveAccount2,
        objectiveDataAddr: roadmapDataAdder.publicKey,
        objectiveVerifiedUser: verifiedUserAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    await program.methods
      .addChildObjective()
      .accounts({
        nameRouterAccount,
        objectiveAccount: objectiveAccount2,
        roadmapMetadataAccount: null,
        childObjectiveAdder: roadmapDataAdder.publicKey,
        objectiveVerifiedUser: verifiedUserAccount,
        parentAccount: objectiveAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });
  });
  it("Add a PR to an issue", async () => {
    //generates key pairs and airdrops solana to them
    const roadmapDataAdder = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [verifiedUserAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      roadmapDataAdder.publicKey
    );

    //adds logs to keypair

    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      preInstructions,
    ] = await create_spl_token(roadmapDataAdder);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([roadmapDataAdder, mintKeypair])
      .rpc();

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
    const issueURI = `https://github.com/${userName}/${repositoryName}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueAccount,
      true
    );

    await program.methods
      .addIssue(issueURI)
      .accounts({
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        issueTokenPoolAccount,
        issueVerifiedUser,
        nameRouterAccount,
        repositoryAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc();

    // Adding a commit
    const treeHash = sha256("Tree hash 1").slice(0, 8);
    const commitHash = sha256("Commit hash 1").slice(0, 8);
    const metadataURI =
      "https://arweave.net/jB7pLq6IReTCeJRHhXiYrfhdEFBeZEDppMc8fkxvJj0";

    const [commitAccount] = await get_pda_from_seeds([
      Buffer.from("commit"),
      Buffer.from(commitHash),
      roadmapDataAdder.publicKey.toBuffer(),
      issueAccount.toBuffer(),
    ]);

    await program.methods
      .addCommit(commitHash, treeHash, metadataURI)
      .accounts({
        commitAccount,
        commitCreator: roadmapDataAdder.publicKey,
        commitVerifiedUser: verifiedUserAccount,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        nameRouterAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    const [pullRequestMetadataAccount] = await get_pda_from_seeds([
      Buffer.from("pullrequestadded"),
      issueAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);

    await program.methods
      .addPr(pull_request_metadata_uri)
      .accounts({
        pullRequestVerifiedUser: verifiedUserAccount,
        issue: issueAccount,
        commit: commitAccount,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        nameRouterAccount,
        pullRequestAddr: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });
  });
  it("Add commit to PR", async () => {
    //generates key pairs and airdrops solana to them
    const roadmapDataAdder = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [verifiedUserAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      roadmapDataAdder.publicKey
    );

    //adds logs to keypair

    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      preInstructions,
    ] = await create_spl_token(roadmapDataAdder);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryCreatorTokenAccount,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([roadmapDataAdder, mintKeypair])
      .rpc();

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
    const issueURI = `https://github.com/${userName}/${repositoryName}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueAccount,
      true
    );

    await program.methods
      .addIssue(issueURI)
      .accounts({
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        issueTokenPoolAccount,
        issueVerifiedUser,
        nameRouterAccount,
        repositoryAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc();

    // Adding a commit
    const treeHash = sha256("Tree hash 1").slice(0, 8);
    const commitHash = sha256("Commit hash 1").slice(0, 8);
    const metadataURI =
      "https://arweave.net/jB7pLq6IReTCeJRHhXiYrfhdEFBeZEDppMc8fkxvJj0";

    const [commitAccount] = await get_pda_from_seeds([
      Buffer.from("commit"),
      Buffer.from(commitHash),
      roadmapDataAdder.publicKey.toBuffer(),
      issueAccount.toBuffer(),
    ]);

    await program.methods
      .addCommit(commitHash, treeHash, metadataURI)
      .accounts({
        commitAccount,
        commitCreator: roadmapDataAdder.publicKey,
        commitVerifiedUser: verifiedUserAccount,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        nameRouterAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    const [pullRequestMetadataAccount] = await get_pda_from_seeds([
      Buffer.from("pullrequestadded"),
      issueAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);

    await program.methods
      .addPr(pull_request_metadata_uri)
      .accounts({
        pullRequestVerifiedUser: verifiedUserAccount,
        issue: issueAccount,
        commit: commitAccount,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        nameRouterAccount,
        pullRequestAddr: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    // Adding a commit
    const treeHash2 = sha256("Tree hash 2").slice(0, 8);
    const commitHash2 = sha256("Commit hash 2").slice(0, 8);
    const metadataURI2 =
      "https://arweave.net/jB7pLq6IReTCeJRHhXiYrfhdEFBeZEDppMc8fkxvJj0";

    const [commitAccount2] = await get_pda_from_seeds([
      Buffer.from("commit"),
      Buffer.from(commitHash2),
      roadmapDataAdder.publicKey.toBuffer(),
      issueAccount.toBuffer(),
    ]);

    await program.methods
      .addCommit(commitHash2, treeHash2, metadataURI2)
      .accounts({
        commitAccount: commitAccount2,
        commitCreator: roadmapDataAdder.publicKey,
        commitVerifiedUser: verifiedUserAccount,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        nameRouterAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    await program.methods
      .addCommitToPr()
      .accounts({
        commitVerifiedUser: verifiedUserAccount,
        commitAddr: roadmapDataAdder.publicKey,
        commit: commitAccount2,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        nameRouterAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });
  });

  it("Accepts a PR", async () => {
    //generates key pairs and airdrops solana to them
    const roadmapDataAdder = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [verifiedUserAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      roadmapDataAdder.publicKey
    );

    //adds logs to keypair

    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      preInstructions,
    ] = await create_spl_token(roadmapDataAdder);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryCreatorTokenAccount,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([roadmapDataAdder, mintKeypair])
      .rpc();

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
    const issueURI = `https://github.com/${userName}/${repositoryName}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueAccount,
      true
    );

    await program.methods
      .addIssue(issueURI)
      .accounts({
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        issueTokenPoolAccount,
        issueVerifiedUser,
        nameRouterAccount,
        repositoryAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc();

    // Adding a commit
    const treeHash = sha256("Tree hash 1").slice(0, 8);
    const commitHash = sha256("Commit hash 1").slice(0, 8);
    const metadataURI =
      "https://arweave.net/jB7pLq6IReTCeJRHhXiYrfhdEFBeZEDppMc8fkxvJj0";

    const [commitAccount] = await get_pda_from_seeds([
      Buffer.from("commit"),
      Buffer.from(commitHash),
      roadmapDataAdder.publicKey.toBuffer(),
      issueAccount.toBuffer(),
    ]);

    await program.methods
      .addCommit(commitHash, treeHash, metadataURI)
      .accounts({
        commitAccount,
        commitCreator: roadmapDataAdder.publicKey,
        commitVerifiedUser: verifiedUserAccount,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        nameRouterAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    const [pullRequestMetadataAccount] = await get_pda_from_seeds([
      Buffer.from("pullrequestadded"),
      issueAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);

    await program.methods
      .addPr(pull_request_metadata_uri)
      .accounts({
        pullRequestVerifiedUser: verifiedUserAccount,
        issue: issueAccount,
        commit: commitAccount,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        nameRouterAccount,
        pullRequestAddr: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    // Adding a commit
    const treeHash2 = sha256("Tree hash 2").slice(0, 8);
    const commitHash2 = sha256("Commit hash 2").slice(0, 8);
    const metadataURI2 =
      "https://arweave.net/jB7pLq6IReTCeJRHhXiYrfhdEFBeZEDppMc8fkxvJj0";

    const [commitAccount2] = await get_pda_from_seeds([
      Buffer.from("commit"),
      Buffer.from(commitHash2),
      roadmapDataAdder.publicKey.toBuffer(),
      issueAccount.toBuffer(),
    ]);

    await program.methods
      .addCommit(commitHash2, treeHash2, metadataURI2)
      .accounts({
        commitAccount: commitAccount2,
        commitCreator: roadmapDataAdder.publicKey,
        commitVerifiedUser: verifiedUserAccount,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        nameRouterAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    await program.methods
      .addCommitToPr()
      .accounts({
        commitVerifiedUser: verifiedUserAccount,
        commitAddr: roadmapDataAdder.publicKey,
        commit: commitAccount2,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        nameRouterAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    await program.methods
      .acceptPr(repositoryName)
      .accounts({
        nameRouterAccount,
        repositoryVerifiedUser: verifiedUserAccount,
        pullRequestAddr: roadmapDataAdder.publicKey,
        pullRequestVerifiedUser: verifiedUserAccount,
        pullRequestMetadataAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryAccount,
        issue: issueAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: false });
  });

  it("Creates a Repository and claims first vesting amount", async () => {
    //generates key pairs and airdrops solana to them
    const roadmapDataAdder = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [verifiedUserAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      roadmapDataAdder.publicKey
    );

    //adds logs to keypair

    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      preInstructions,
    ] = await create_spl_token(roadmapDataAdder);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryCreatorTokenAccount,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([roadmapDataAdder, mintKeypair])
      .rpc();

    await program.methods
      .unlockTokens(repositoryName)
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        tokenMint: mintKeypair.publicKey,
        vestingTokenAccount: vestingTokenAccount,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: false });
  });

  it("Getting a PR accepted and getting rewarded for it", async () => {
    //generates key pairs and airdrops solana to them
    const roadmapDataAdder = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [verifiedUserAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      roadmapDataAdder.publicKey
    );

    //adds logs to keypair

    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      preInstructions,
    ] = await create_spl_token(roadmapDataAdder);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryCreatorTokenAccount,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([roadmapDataAdder, mintKeypair])
      .rpc();

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
    const issueURI = `https://github.com/${userName}/${repositoryName}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueAccount,
      true
    );

    await program.methods
      .addIssue(issueURI)
      .accounts({
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        issueTokenPoolAccount,
        issueVerifiedUser,
        nameRouterAccount,
        repositoryAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc();

    // Adding a commit
    const treeHash = sha256("Tree hash 1").slice(0, 8);
    const commitHash = sha256("Commit hash 1").slice(0, 8);
    const metadataURI =
      "https://arweave.net/jB7pLq6IReTCeJRHhXiYrfhdEFBeZEDppMc8fkxvJj0";

    const [commitAccount] = await get_pda_from_seeds([
      Buffer.from("commit"),
      Buffer.from(commitHash),
      roadmapDataAdder.publicKey.toBuffer(),
      issueAccount.toBuffer(),
    ]);

    await program.methods
      .addCommit(commitHash, treeHash, metadataURI)
      .accounts({
        commitAccount,
        commitCreator: roadmapDataAdder.publicKey,
        commitVerifiedUser: verifiedUserAccount,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        nameRouterAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    const [pullRequestMetadataAccount] = await get_pda_from_seeds([
      Buffer.from("pullrequestadded"),
      issueAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);

    await program.methods
      .addPr(pull_request_metadata_uri)
      .accounts({
        pullRequestVerifiedUser: verifiedUserAccount,
        issue: issueAccount,
        commit: commitAccount,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        nameRouterAccount,
        pullRequestAddr: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    // Adding a commit
    const treeHash2 = sha256("Tree hash 2").slice(0, 8);
    const commitHash2 = sha256("Commit hash 2").slice(0, 8);
    const metadataURI2 =
      "https://arweave.net/jB7pLq6IReTCeJRHhXiYrfhdEFBeZEDppMc8fkxvJj0";

    const [commitAccount2] = await get_pda_from_seeds([
      Buffer.from("commit"),
      Buffer.from(commitHash2),
      roadmapDataAdder.publicKey.toBuffer(),
      issueAccount.toBuffer(),
    ]);

    await program.methods
      .addCommit(commitHash2, treeHash2, metadataURI2)
      .accounts({
        commitAccount: commitAccount2,
        commitCreator: roadmapDataAdder.publicKey,
        commitVerifiedUser: verifiedUserAccount,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        nameRouterAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    await program.methods
      .addCommitToPr()
      .accounts({
        commitVerifiedUser: verifiedUserAccount,
        commitAddr: roadmapDataAdder.publicKey,
        commit: commitAccount2,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        nameRouterAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: true });

    const pullRequestCreatorRewardAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      roadmapDataAdder.publicKey
    );

    // Staking tokens on a issue
    const issueStakerKeypair = await create_keypair();
    const transferAmount = 1000 * 10 ** 9;
    const issueStakerTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      issueStakerKeypair.publicKey
    );

    const createIssueStakerTokenAccountIx =
      createAssociatedTokenAccountInstruction(
        issueStakerKeypair.publicKey,
        issueStakerTokenAccount,
        issueStakerKeypair.publicKey,
        mintKeypair.publicKey
      );

    const mintToIssueStakerIx = createMintToCheckedInstruction(
      mintKeypair.publicKey,
      issueStakerTokenAccount,
      roadmapDataAdder.publicKey,
      transferAmount,
      9,
      []
    );

    const [issueStakerAccount] = await get_pda_from_seeds([
      Buffer.from("issuestaker"),
      issueAccount.toBuffer(),
      issueStakerKeypair.publicKey.toBuffer(),
    ]);

    await program.methods
      .stakeIssue(new anchor.BN(10))
      .accounts({
        issueAccount,
        repositoryAccount,
        issueTokenPoolAccount,
        issueStaker: issueStakerKeypair.publicKey,
        issueStakerAccount,
        issueStakerTokenAccount,
        rewardsMint: mintKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .preInstructions([createIssueStakerTokenAccountIx, mintToIssueStakerIx])
      .signers([roadmapDataAdder, issueStakerKeypair])
      .rpc();

    await program.methods
      .acceptPr(repositoryName)
      .accounts({
        nameRouterAccount,
        repositoryVerifiedUser: verifiedUserAccount,
        pullRequestAddr: roadmapDataAdder.publicKey,
        pullRequestVerifiedUser: verifiedUserAccount,
        pullRequestMetadataAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryAccount,
        issue: issueAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: false });

    await program.methods
      .claimReward()
      .accounts({
        nameRouterAccount,
        repositoryVerifiedUser: verifiedUserAccount,
        pullRequestCreator: roadmapDataAdder.publicKey,
        pullRequestVerifiedUser: verifiedUserAccount,
        pullRequest: pullRequestMetadataAccount,
        pullRequestCreatorRewardAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        rewardsMint: mintKeypair.publicKey,
        repositoryAccount,
        issueAccount: issueAccount,
        issueTokenPoolAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc({ skipPreflight: false });
  });

  it("Create communal account to store tokens",async() => {
    //generates key pairs and airdrops solana to them
    const repositoryCreator = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [verifiedUserAccount] = await create_verified_user(
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
      preInstructions,
    ] = await create_spl_token(repositoryCreator);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([repositoryCreator, mintKeypair])
      .rpc({ skipPreflight: false });

    const [communal_account] = await get_pda_from_seeds(
      [
        Buffer.from("are_we_conscious"),
        Buffer.from("is love life ?  "),
        Buffer.from("arewemadorinlove"),
        mintKeypair.publicKey.toBuffer()
    ]
    );

    const communalTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      communal_account,
      true
    )

    await program.methods
         .createCommunalAccount()
         .accounts({
          authority:repositoryCreator.publicKey,
          communalDeposit: communal_account,
          communalTokenAccount:communalTokenAccount,
          systemProgram:web3.SystemProgram.programId,
          rewardsMint:mintKeypair.publicKey,
          tokenProgram:TOKEN_PROGRAM_ID,
          associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID
         })
         .signers([repositoryCreator])
         .rpc({skipPreflight:true})
  })

  it("Sends a buy transaction",async() => {
    //generates key pairs and airdrops solana to them
    const repositoryCreator = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [verifiedUserAccount] = await create_verified_user(
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
      preInstructions,
    ] = await create_spl_token(repositoryCreator);

    await program.methods
      .createRepository(
        repositoryName,
        "Open source revolution",
        "https://github.com/sunguru98/defios"
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
      })
      .preInstructions(preInstructions)
      .signers([repositoryCreator, mintKeypair])
      .rpc({ skipPreflight: false });

    const [communal_account] = await get_pda_from_seeds(
      [
        Buffer.from("are_we_conscious"),
        Buffer.from("is love life ?  "),
        Buffer.from("arewemadorinlove"),
        mintKeypair.publicKey.toBuffer()
    ]
    );

    const communalTokenAccount = await getAssociatedTokenAddress(
      mintKeypair.publicKey,
      communal_account,
      true
    )

    await program.methods
         .createCommunalAccount()
         .accounts({
          authority:repositoryCreator.publicKey,
          communalDeposit: communal_account,
          communalTokenAccount:communalTokenAccount,
          systemProgram:web3.SystemProgram.programId,
          rewardsMint:mintKeypair.publicKey,
          tokenProgram:TOKEN_PROGRAM_ID,
          associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID
         })
         .signers([repositoryCreator])
         .rpc({skipPreflight:true})
    
    await program.methods
         .buyTokens(new anchor.BN(1))
         .accounts({
          buyer:repositoryCreator.publicKey,
          communalDeposit:communal_account,
          communalTokenAccount:communalTokenAccount,
          rewardsMint:mintKeypair.publicKey,
          tokenProgram:TOKEN_PROGRAM_ID,
          associatedTokenProgram:ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram:web3.SystemProgram.programId,
          buyerTokenAccount:repositoryCreatorTokenAccount
         })
         .signers([repositoryCreator])
         .rpc({skipPreflight:false})
  })
});
