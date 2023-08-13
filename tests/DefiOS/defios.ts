import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Defios } from "../../target/types/defios";

import * as ed from "@noble/ed25519";
import { PublicKey } from "@saberhq/solana-contrib";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Metaplex } from "@metaplex-foundation/js";
import sha256 from "sha256";
import { rpcConfig } from "../test_config";

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
  //global variables for tests
  const signatureVersion = 1;
  const signingName = "defios.com";
  const userName: string = "sunguru98";
  const userPubkey = new PublicKey(
    "81sWMLg1EgYps3nMwyeSW1JfjKgFqkGYPP85vTnkFzRn"
  );
  const repositoryId = "12";
  const roadmapTitle = "Test Roadmap";
  const roadmapImageUrl = "https://github.com/defi-os/Issues";
  const roadmapDescription = "https://github.com/defi-os/Issues";
  const roadmapOutlook = { next2: {} };
  const objectiveDeliverable = { tooling: {} };
  const objectiveTitle = "Test Objective";
  const objectiveDescription = "https://github.com/defi-os/Issues";
  const objectiveEndUnix = new anchor.BN(1735603200);
  const objectiveStartUnix = new anchor.BN(1704067200);
  const pull_request_metadata_uri = "https://github.com/defi-os/Issues";
  const new_schedule = [
    {
      releaseTime: new anchor.BN(1),
      amount: new anchor.BN(1),
    },
    {
      releaseTime: new anchor.BN(1),
      amount: new anchor.BN(1),
    },
  ];
  const tokenName = "Hi!";
  const tokenimage = "BRR";
  const tokenMetadata =
    "https://en.wikipedia.org/wiki/File:Bonnet_macaque_(Macaca_radiata)_Photograph_By_Shantanu_Kuveskar.jpg";
  const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
  );
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

  async function get_metadata_account(mintKeypair) {
    return (
      await anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("metadata"),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          mintKeypair.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID
      )
    )[0];
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
      .rpc(rpcConfig);
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
      .rpc(rpcConfig);
    return [verifiedUserAccount];
  }

  async function create_spl_token(repositoryCreator) {
    // Creating repository
    const [repositoryAccount] = await get_pda_from_seeds([
      Buffer.from("repository"),
      Buffer.from(repositoryId),
      repositoryCreator.publicKey.toBuffer(),
    ]);

    // Creating rewards mint
    const [mint] = await get_pda_from_seeds([
      Buffer.from("Miners"),
      Buffer.from("MinerC"),
      repositoryAccount.toBuffer(),
    ]);

    const [vestingAccount] = await get_pda_from_seeds([
      Buffer.from("vesting"),
      repositoryAccount.toBuffer(),
    ]);

    const vestingTokenAccount = await getAssociatedTokenAddress(
      mint,
      vestingAccount,
      true
    );

    const repositoryCreatorTokenAccount = await getAssociatedTokenAddress(
      mint,
      repositoryCreator.publicKey
    );

    const [defaultVestingSchedule] = await get_pda_from_seeds([
      Buffer.from("isGodReal?"),
      Buffer.from("DoULoveMe?"),
      Buffer.from("SweetChick"),
    ]);

    await program.methods
      .setDefaultSchedule(4, new anchor.BN(2500), new anchor.BN(10 ** 7))
      .accounts({
        authority: repositoryCreator.publicKey,
        defaultSchedule: defaultVestingSchedule,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

    return [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mint,
      vestingAccount,
      defaultVestingSchedule,
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

  it("Creates a repository with new spl token", async () => {
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
      defaultVestingSchedule,
    ] = await create_spl_token(repositoryCreator);

    const metadataAddress = await get_metadata_account(mintKeypair);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });

  it("Creates a repository without new spl token", async () => {
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
      defaultVestingSchedule,
    ] = await create_spl_token(repositoryCreator);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        null,
        null,
        null
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount: null,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: null,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: null,
        vestingTokenAccount: null,
        defaultSchedule: defaultVestingSchedule,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        metadata: null,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
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
      defaultVestingSchedule,
    ] = await create_spl_token(repositoryCreator);

    const metadataAddress = await get_metadata_account(mintKeypair);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: repositoryVerifiedUser,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

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
    const issueURI = `https://github.com/${userName}/${repositoryId}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
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
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: repositoryCreator.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc(rpcConfig);
  });

  it("Stakes on a issue", async () => {
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const repositoryCreator = await create_keypair();
    const issueCreatorKeypair = await create_keypair();
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
      defaultVestingSchedule,
    ] = await create_spl_token(repositoryCreator);

    const metadataAddress = await get_metadata_account(mintKeypair);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: repositoryVerifiedUser,
        rewardsMint: mintKeypair,
        defaultSchedule: defaultVestingSchedule,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        metadata: metadataAddress,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

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
    const issueURI = `https://github.com/${userName}/${repositoryId}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
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
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: repositoryCreator.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc(rpcConfig);

    await program.account.issue.fetch(issueAccount);

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
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });

  it("Unstakes on a issue", async () => {
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const repositoryCreator = await create_keypair();
    const issueCreatorKeypair = await create_keypair();

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
      defaultVestingSchedule,
    ] = await create_spl_token(repositoryCreator);

    // Creating repository
    const metadataAddress = await get_metadata_account(mintKeypair);
    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: repositoryVerifiedUser,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        metadata: metadataAddress,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

    const { issueIndex } = await program.account.repository.fetch(
      repositoryAccount
    );

    const [issueVerifiedUser] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      issueCreatorKeypair.publicKey
    );

    // Creating issue
    const issueURI = `https://github.com/${userName}/${repositoryId}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
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
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: repositoryCreator.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc(rpcConfig);

    await program.account.issue.fetch(issueAccount);

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
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

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
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });

  it("Adds a commit to an issue", async () => {
    const repositoryCreator = await create_keypair();
    const issueCreatorKeypair = await create_keypair();
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
      defaultVestingSchedule,
    ] = await create_spl_token(repositoryCreator);
    const metadataAddress = await get_metadata_account(mintKeypair);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: repositoryVerifiedUser,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

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
    const issueURI = `https://github.com/${userName}/${repositoryId}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
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
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: repositoryCreator.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc(rpcConfig);

    await program.account.issue.fetch(issueAccount);

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
      .rpc(rpcConfig);
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
    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      defaultVestingSchedule,
    ] = await create_spl_token(roadmapDataAdder);

    const metadataAddress = await get_metadata_account(mintKeypair);
    const [metadataAccount] = await get_pda_from_seeds([
      Buffer.from("roadmapmetadataadd"),
      repositoryAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

    await program.methods
      .addRoadmapData(
        roadmapTitle,
        roadmapDescription,
        roadmapImageUrl,
        roadmapOutlook
      )
      .accounts({
        nameRouterAccount,
        metadataAccount,
        roadmapDataAdder: roadmapDataAdder.publicKey,
        roadmapVerifiedUser: verifiedUserAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryAccount: repositoryAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);
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
      defaultVestingSchedule,
    ] = await create_spl_token(roadmapDataAdder);
    const metadataAddress = await get_metadata_account(mintKeypair);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

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
    const issueURI = `https://github.com/${userName}/${repositoryId}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
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
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc(rpcConfig);

    const [metadataAccount] = await get_pda_from_seeds([
      Buffer.from("roadmapmetadataadd"),
      repositoryAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);
    await program.methods
      .addRoadmapData(
        roadmapTitle,
        roadmapDescription,
        roadmapImageUrl,
        roadmapOutlook
      )
      .accounts({
        nameRouterAccount,
        metadataAccount,
        roadmapDataAdder: roadmapDataAdder.publicKey,
        roadmapVerifiedUser: verifiedUserAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryAccount: repositoryAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);
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
        repositoryAccount: repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);
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
      defaultVestingSchedule,
    ] = await create_spl_token(roadmapDataAdder);

    const metadataAddress = await get_metadata_account(mintKeypair);
    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

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
    const issueURI = `https://github.com/${userName}/${repositoryId}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
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
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc(rpcConfig);

    const [metadataAccount] = await get_pda_from_seeds([
      Buffer.from("roadmapmetadataadd"),
      repositoryAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);
    await program.methods
      .addRoadmapData(
        roadmapTitle,
        roadmapDescription,
        roadmapImageUrl,
        roadmapOutlook
      )
      .accounts({
        nameRouterAccount,
        metadataAccount,
        roadmapDataAdder: roadmapDataAdder.publicKey,
        roadmapVerifiedUser: verifiedUserAccount,
        repositoryAccount: repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

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
        repositoryAccount: repositoryAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

    await program.methods
      .addChildObjective()
      .accounts({
        roadmapMetadataAccount: metadataAccount,
        childObjectiveAdder: roadmapDataAdder.publicKey,
        parentObjectiveAccount: null,
        systemProgram: web3.SystemProgram.programId,
      })
      .remainingAccounts([
        { pubkey: objectiveAccount, isWritable: true, isSigner: false },
      ])
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);
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
      defaultVestingSchedule,
    ] = await create_spl_token(roadmapDataAdder);

    const metadataAddress = await get_metadata_account(mintKeypair);
    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

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
    const issueURI = `https://github.com/${userName}/${repositoryId}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
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
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc(rpcConfig);

    const [metadataAccount] = await get_pda_from_seeds([
      Buffer.from("roadmapmetadataadd"),
      repositoryAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);
    await program.methods
      .addRoadmapData(
        roadmapTitle,
        roadmapDescription,
        roadmapImageUrl,
        roadmapOutlook
      )
      .accounts({
        nameRouterAccount,
        metadataAccount,
        roadmapDataAdder: roadmapDataAdder.publicKey,
        roadmapVerifiedUser: verifiedUserAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryAccount: repositoryAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

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
        repositoryAccount: repositoryAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

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
        repositoryAccount: repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

    await program.methods
      .addChildObjective()
      .accounts({
        roadmapMetadataAccount: null,
        childObjectiveAdder: roadmapDataAdder.publicKey,
        objectiveVerifiedUser: verifiedUserAccount,
        parentObjectiveAccount: objectiveAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .remainingAccounts([
        { pubkey: objectiveAccount, isWritable: true, isSigner: false },
      ])
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);
  });
  it("Adds a PR to an issue", async () => {
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
      defaultVestingSchedule,
    ] = await create_spl_token(roadmapDataAdder);
    const metadataAddress = await get_metadata_account(mintKeypair);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

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
    const issueURI = `https://github.com/${userName}/${repositoryId}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
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
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc(rpcConfig);

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
      .rpc(rpcConfig);

    const [pullRequestMetadataAccount] = await get_pda_from_seeds([
      Buffer.from("pullrequestadded"),
      issueAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);

    const pullRequestTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      pullRequestMetadataAccount,
      true
    );

    await program.methods
      .addPr(pull_request_metadata_uri)
      .accounts({
        pullRequestVerifiedUser: verifiedUserAccount,
        issue: issueAccount,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        nameRouterAccount,
        pullRequestTokenAccount,
        pullRequestAddr: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rewardsMint: mintKeypair,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .remainingAccounts([
        { pubkey: commitAccount, isWritable: true, isSigner: false },
      ])
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);
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
      defaultVestingSchedule,
    ] = await create_spl_token(roadmapDataAdder);

    const metadataAddress = await get_metadata_account(mintKeypair);
    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryCreatorTokenAccount,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

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
    const issueURI = `https://github.com/${userName}/${repositoryId}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
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
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc(rpcConfig);

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
      .rpc(rpcConfig);

    const [pullRequestMetadataAccount] = await get_pda_from_seeds([
      Buffer.from("pullrequestadded"),
      issueAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);

    const pullRequestTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      pullRequestMetadataAccount,
      true
    );

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
      .rpc(rpcConfig);

    await program.methods
      .addPr(pull_request_metadata_uri)
      .accounts({
        pullRequestVerifiedUser: verifiedUserAccount,
        issue: issueAccount,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        nameRouterAccount,
        pullRequestTokenAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        rewardsMint: mintKeypair,
        pullRequestAddr: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .remainingAccounts([
        { pubkey: commitAccount, isWritable: true, isSigner: false },
      ])
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

    await program.methods
      .addCommitToPr()
      .accounts({
        commitAddr: roadmapDataAdder.publicKey,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .remainingAccounts([
        { pubkey: commitAccount2, isWritable: true, isSigner: false },
      ])
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);
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
      defaultVestingSchedule,
    ] = await create_spl_token(roadmapDataAdder);
    const metadataAddress = await get_metadata_account(mintKeypair);
    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryCreatorTokenAccount,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

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
    const issueURI = `https://github.com/${userName}/${repositoryId}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
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
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc(rpcConfig);

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
      .rpc(rpcConfig);

    const [pullRequestMetadataAccount] = await get_pda_from_seeds([
      Buffer.from("pullrequestadded"),
      issueAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);

    const pullRequestTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      pullRequestMetadataAccount,
      true
    );

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
      .rpc(rpcConfig);

    await program.methods
      .addPr(pull_request_metadata_uri)
      .accounts({
        pullRequestVerifiedUser: verifiedUserAccount,
        issue: issueAccount,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        nameRouterAccount,
        pullRequestTokenAccount,
        pullRequestAddr: roadmapDataAdder.publicKey,
        repositoryAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .remainingAccounts([
        { pubkey: commitAccount, isWritable: true, isSigner: false },
        { pubkey: commitAccount2, isWritable: true, isSigner: false },
      ])
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

    await program.methods
      .acceptPr(repositoryId)
      .accounts({
        pullRequestAddr: roadmapDataAdder.publicKey,
        pullRequestMetadataAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryAccount,
        issue: issueAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);
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
      defaultVestingSchedule,
    ] = await create_spl_token(roadmapDataAdder);
    const metadataAddress = await get_metadata_account(mintKeypair);
    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryCreatorTokenAccount,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        metadata: metadataAddress,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

    await program.methods
      .unlockTokens()
      .accounts({
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        tokenMint: mintKeypair,
        vestingTokenAccount: vestingTokenAccount,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);
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
      defaultVestingSchedule,
    ] = await create_spl_token(roadmapDataAdder);
    const metadataAddress = await get_metadata_account(mintKeypair);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryCreatorTokenAccount,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

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
    const issueURI = `https://github.com/${userName}/${repositoryId}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
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
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc(rpcConfig);

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
      .rpc(rpcConfig);

    const [pullRequestMetadataAccount] = await get_pda_from_seeds([
      Buffer.from("pullrequestadded"),
      issueAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);

    const pullRequestTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      pullRequestMetadataAccount,
      true
    );

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
      .rpc(rpcConfig);

    await program.methods
      .addPr(pull_request_metadata_uri)
      .accounts({
        pullRequestVerifiedUser: verifiedUserAccount,
        issue: issueAccount,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        nameRouterAccount,
        pullRequestTokenAccount,
        rewardsMint: mintKeypair,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        pullRequestAddr: roadmapDataAdder.publicKey,
        repositoryAccount,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .remainingAccounts([
        { pubkey: commitAccount, isWritable: true, isSigner: false },
        { pubkey: commitAccount2, isWritable: true, isSigner: false },
      ])
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

    const pullRequestCreatorRewardAccount = await getAssociatedTokenAddress(
      mintKeypair,
      roadmapDataAdder.publicKey
    );

    const [issueStakerAccount] = await get_pda_from_seeds([
      Buffer.from("issuestaker"),
      issueAccount.toBuffer(),
      roadmapDataAdder.publicKey.toBuffer(),
    ]);

    await program.methods
      .unlockTokens()
      .accounts({
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        tokenMint: mintKeypair,
        vestingTokenAccount: vestingTokenAccount,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

    await program.methods
      .stakeIssue(new anchor.BN(10))
      .accounts({
        issueAccount,
        repositoryAccount,
        issueTokenPoolAccount,
        issueStaker: roadmapDataAdder.publicKey,
        issueStakerAccount,
        issueStakerTokenAccount: repositoryCreatorTokenAccount,
        rewardsMint: mintKeypair,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

    await program.methods
      .acceptPr(repositoryId)
      .accounts({
        pullRequestAddr: roadmapDataAdder.publicKey,
        pullRequestMetadataAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        repositoryAccount,
        issue: issueAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);

    await program.methods
      .claimReward()
      .accounts({
        pullRequestCreator: roadmapDataAdder.publicKey,
        pullRequest: pullRequestMetadataAccount,
        pullRequestCreatorRewardAccount,
        repositoryCreator: roadmapDataAdder.publicKey,
        rewardsMint: mintKeypair,
        repositoryAccount,
        issueAccount: issueAccount,
        issueTokenPoolAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([roadmapDataAdder])
      .rpc(rpcConfig);
  });

  it("Create communal account to store tokens", async () => {
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
      defaultVestingSchedule,
    ] = await create_spl_token(repositoryCreator);
    const metadataAddress = await get_metadata_account(mintKeypair);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

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
  });

  it("Sends a buy transaction", async () => {
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
      defaultVestingSchedule,
    ] = await create_spl_token(repositoryCreator);
    const metadataAddress = await get_metadata_account(mintKeypair);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

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
      .buyTokens(new anchor.BN(1), new anchor.BN(1))
      .accounts({
        buyer: repositoryCreator.publicKey,
        communalDeposit: communal_account,
        communalTokenAccount: communalTokenAccount,
        rewardsMint: mintKeypair,
        tokenProgram: TOKEN_PROGRAM_ID,
        repositoryAccount: repositoryAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
        buyerTokenAccount: repositoryCreatorTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        communalUsdcAccount: communalTokenAccount,
        buyerUsdcAccount: repositoryCreatorTokenAccount,
        usdcMint: mintKeypair,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });

  it("Sends a sell transaction", async () => {
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
      defaultVestingSchedule,
    ] = await create_spl_token(repositoryCreator);
    const metadataAddress = await get_metadata_account(mintKeypair);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        metadata: metadataAddress,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

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
      .sellTokens(new anchor.BN(0), new anchor.BN(1))
      .accounts({
        seller: repositoryCreator.publicKey,
        communalDeposit: communal_account,
        communalTokenAccount: communalTokenAccount,
        rewardsMint: mintKeypair,
        repositoryAccount: repositoryAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
        sellerTokenAccount: repositoryCreatorTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        communalUsdcAccount: communalTokenAccount,
        sellerUsdcAccount: repositoryCreatorTokenAccount,
        usdcMint: mintKeypair,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });

  it("Custom SPL Token integration test", async () => {
    //generates key pairs and airdrops solana to them
    const repositoryCreator = await create_keypair();
    const repositoryCreator2 = await create_keypair();
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();
    const [verifiedUserAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      repositoryCreator.publicKey
    );

    const [verifiedUserAccount2] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      repositoryCreator2.publicKey
    );

    const [
      repositoryAccount,
      repositoryCreatorTokenAccount,
      vestingTokenAccount,
      mintKeypair,
      vestingAccount,
      defaultVestingSchedule,
    ] = await create_spl_token(repositoryCreator);
    const metadataAddress = await get_metadata_account(mintKeypair);
    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        metadata: metadataAddress,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

    const [
      repositoryAccount2,
      repositoryCreatorTokenAccount2,
      vestingTokenAccount2,
      mintKeypair2,
      vestingAccount2,
      defaultVestingSchedule2,
    ] = await create_spl_token(repositoryCreator2);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        null,
        null,
        null
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount2,
        repositoryCreatorTokenAccount: null,
        repositoryCreator: repositoryCreator2.publicKey,
        repositoryVerifiedUser: verifiedUserAccount2,
        rewardsMint: null,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: null,
        vestingTokenAccount: null,
        defaultSchedule: defaultVestingSchedule,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        metadata: null,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator2])
      .rpc(rpcConfig);

    const issueCreatorKeypair = await create_keypair();

    const { issueIndex } = await program.account.repository.fetch(
      repositoryAccount2
    );

    // Adding issue creator user

    const [issueVerifiedUser] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      issueCreatorKeypair.publicKey
    );
    // Creating issue
    const issueURI = `https://github.com/${userName}/${repositoryId}/issues/${issueIndex}`;
    const [issueAccount] = await get_pda_from_seeds([
      Buffer.from("issue"),
      Buffer.from(issueIndex.toString()),
      repositoryAccount2.toBuffer(),
      issueCreatorKeypair.publicKey.toBuffer(),
    ]);

    const issueTokenPoolAccount = await getAssociatedTokenAddress(
      mintKeypair,
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
        repositoryAccount: repositoryAccount2,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        repositoryCreator: repositoryCreator2.publicKey,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([issueCreatorKeypair])
      .rpc(rpcConfig);

    // Adding a commit
    const treeHash = sha256("Tree hash 1").slice(0, 8);
    const commitHash = sha256("Commit hash 1").slice(0, 8);
    const metadataURI =
      "https://arweave.net/jB7pLq6IReTCeJRHhXiYrfhdEFBeZEDppMc8fkxvJj0";

    const [commitAccount] = await get_pda_from_seeds([
      Buffer.from("commit"),
      Buffer.from(commitHash),
      repositoryCreator2.publicKey.toBuffer(),
      issueAccount.toBuffer(),
    ]);

    await program.methods
      .addCommit(commitHash, treeHash, metadataURI)
      .accounts({
        commitAccount,
        commitCreator: repositoryCreator2.publicKey,
        commitVerifiedUser: verifiedUserAccount2,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        nameRouterAccount,
        repositoryCreator: repositoryCreator2.publicKey,
        repositoryAccount: repositoryAccount2,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([repositoryCreator2])
      .rpc(rpcConfig);

    const [pullRequestMetadataAccount] = await get_pda_from_seeds([
      Buffer.from("pullrequestadded"),
      issueAccount.toBuffer(),
      repositoryCreator2.publicKey.toBuffer(),
    ]);

    const pullRequestTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      pullRequestMetadataAccount,
      true
    );

    // Adding a commit
    const treeHash2 = sha256("Tree hash 2").slice(0, 8);
    const commitHash2 = sha256("Commit hash 2").slice(0, 8);
    const metadataURI2 =
      "https://arweave.net/jB7pLq6IReTCeJRHhXiYrfhdEFBeZEDppMc8fkxvJj0";

    const [commitAccount2] = await get_pda_from_seeds([
      Buffer.from("commit"),
      Buffer.from(commitHash2),
      repositoryCreator2.publicKey.toBuffer(),
      issueAccount.toBuffer(),
    ]);

    await program.methods
      .addCommit(commitHash2, treeHash2, metadataURI2)
      .accounts({
        commitAccount: commitAccount2,
        commitCreator: repositoryCreator2.publicKey,
        commitVerifiedUser: verifiedUserAccount2,
        issueAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        nameRouterAccount,
        repositoryCreator: repositoryCreator2.publicKey,
        repositoryAccount: repositoryAccount2,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([repositoryCreator2])
      .rpc(rpcConfig);

    await program.methods
      .addPr(pull_request_metadata_uri)
      .accounts({
        pullRequestVerifiedUser: verifiedUserAccount2,
        issue: issueAccount,
        pullRequestMetadataAccount: pullRequestMetadataAccount,
        nameRouterAccount,
        pullRequestTokenAccount,
        rewardsMint: mintKeypair,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        pullRequestAddr: repositoryCreator2.publicKey,
        repositoryAccount: repositoryAccount2,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .remainingAccounts([
        { pubkey: commitAccount, isWritable: true, isSigner: false },
        { pubkey: commitAccount2, isWritable: true, isSigner: false },
      ])
      .signers([repositoryCreator2])
      .rpc(rpcConfig);

    const pullRequestCreatorRewardAccount = await getAssociatedTokenAddress(
      mintKeypair,
      repositoryCreator2.publicKey
    );

    // Staking tokens on a issue
    const issueStakerKeypair = await create_keypair();
    const issueStakerTokenAccount = await getAssociatedTokenAddress(
      mintKeypair,
      issueStakerKeypair.publicKey
    );

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
        repositoryAccount: repositoryAccount2,
        issueTokenPoolAccount,
        issueStaker: repositoryCreator.publicKey,
        issueStakerAccount,
        issueStakerTokenAccount: repositoryCreatorTokenAccount,
        rewardsMint: mintKeypair,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

    await program.methods
      .acceptPr(repositoryId)
      .accounts({
        pullRequestAddr: repositoryCreator2.publicKey,
        pullRequestMetadataAccount,
        repositoryCreator: repositoryCreator2.publicKey,
        repositoryAccount: repositoryAccount2,
        issue: issueAccount,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([repositoryCreator2])
      .rpc(rpcConfig);

    await program.methods
      .claimReward()
      .accounts({
        nameRouterAccount,
        repositoryVerifiedUser: verifiedUserAccount2,
        pullRequestCreator: repositoryCreator2.publicKey,
        pullRequestVerifiedUser: verifiedUserAccount2,
        pullRequest: pullRequestMetadataAccount,
        pullRequestCreatorRewardAccount,
        repositoryCreator: repositoryCreator2.publicKey,
        rewardsMint: mintKeypair,
        repositoryAccount: repositoryAccount2,
        issueAccount: issueAccount,
        issueTokenPoolAccount,
        issueCreator: issueCreatorKeypair.publicKey,
        routerCreator: routerCreatorKeypair.publicKey,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([repositoryCreator2])
      .rpc(rpcConfig);
  });
  it("Change vesting schedule", async () => {
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
      defaultVestingSchedule,
    ] = await create_spl_token(repositoryCreator);
    const metadataAddress = await get_metadata_account(mintKeypair);

    await program.methods
      .createRepository(
        repositoryId,
        "Open source revolution",
        "https://github.com/sunguru98/defios",
        tokenName,
        tokenimage,
        tokenMetadata
      )
      .accounts({
        nameRouterAccount,
        repositoryAccount,
        repositoryCreatorTokenAccount,
        repositoryCreator: repositoryCreator.publicKey,
        repositoryVerifiedUser: verifiedUserAccount,
        rewardsMint: mintKeypair,
        routerCreator: routerCreatorKeypair.publicKey,
        systemProgram: web3.SystemProgram.programId,
        vestingAccount: vestingAccount,
        vestingTokenAccount: vestingTokenAccount,
        defaultSchedule: defaultVestingSchedule,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        metadata: metadataAddress,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);

    const [vestingSchedule] = await get_pda_from_seeds([
      Buffer.from("vesting"),
      repositoryAccount.toBuffer(),
    ]);

    await program.methods
      .changeVestingSchedule(new_schedule)
      .accounts({
        repositoryAccount: repositoryAccount,
        systemProgram: web3.SystemProgram.programId,
        authority: repositoryCreator.publicKey,
        vestingSchedule: vestingSchedule,
      })
      .signers([repositoryCreator])
      .rpc(rpcConfig);
  });
});
