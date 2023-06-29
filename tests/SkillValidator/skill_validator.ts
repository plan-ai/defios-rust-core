import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SkillValidator } from "../../target/types/skill_validator";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { PublicKey } from "@saberhq/solana-contrib";
import * as ed from "@noble/ed25519";

describe("skill_validator", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  //testing defios workspace here
  const program = anchor.workspace.SkillValidator as Program<SkillValidator>;
  const {
    provider: { connection },
  } = program;
  const { web3 } = anchor;
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

  const jobLength = { longTerm: {} };
  const jobName: string = "Solana dev";
  const jobDesc = "Build smart contracts";
  const jobMetadataUri = "https://github.com/defi-os/Issues";
  const stakeAmmount = new anchor.BN(100);
  const mintAmount = 200;
  const signingName = "defios.com";
  const signatureVersion = 1;
  const userMetadataUri = "http:??";
  const userPubkey = new PublicKey(
    "81sWMLg1EgYps3nMwyeSW1JfjKgFqkGYPP85vTnkFzRn"
  );
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
      .addNameRouter(signingName, signatureVersion)
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
      Buffer.from(`DefiOS(${userMetadataUri}, ${userPubkey.toString()})`)
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
      Buffer.from(userMetadataUri),
      pubKey.toBuffer(),
      nameRouterAccount.toBuffer(),
    ]);

    //calls add verified user method
    await program.methods
      .addVerifiedFreelancer(
        userMetadataUri,
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

  it("Create a job posting", async () => {
    const jobPoster = await create_keypair();
    const [job] = await get_pda_from_seeds([
      Buffer.from("boringlif"),
      jobPoster.publicKey.toBuffer(),
      Buffer.from(jobName),
    ]);
    await program.methods
      .addJob(jobName, jobDesc, jobLength, jobMetadataUri)
      .accounts({
        jobAddr: jobPoster.publicKey,
        job: job,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([jobPoster])
      .rpc({ skipPreflight: true });
  });
  it("Stake a job posting", async () => {
    const jobPoster = await create_keypair();
    const mintAuthority = await create_keypair();
    const [job] = await get_pda_from_seeds([
      Buffer.from("boringlif"),
      jobPoster.publicKey.toBuffer(),
      Buffer.from(jobName),
    ]);
    await program.methods
      .addJob(jobName, jobDesc, jobLength, jobMetadataUri)
      .accounts({
        jobAddr: jobPoster.publicKey,
        job: job,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([jobPoster])
      .rpc({ skipPreflight: false });

    //creating spl token
    const mintAddress = await createMint(
      connection,
      mintAuthority,
      mintAuthority.publicKey,
      mintAuthority.publicKey,
      6
    );
    const jobPosterTokenAddress = await createAssociatedTokenAccount(
      connection,
      jobPoster,
      mintAddress,
      jobPoster.publicKey
    );
    const jobTokenAddress = await getAssociatedTokenAddress(
      mintAddress,
      job,
      true
    );
    await mintTo(
      connection,
      jobPoster,
      mintAddress,
      jobPosterTokenAddress,
      mintAuthority,
      mintAmount
    );
    await program.methods
      .stakeJob(stakeAmmount)
      .accounts({
        jobAddr: jobPoster.publicKey,
        job: job,
        jobAddrUsdcAccount: jobPosterTokenAddress,
        jobUsdcAccount: jobTokenAddress,
        usdcMint: mintAddress,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([jobPoster])
      .rpc({ skipPreflight: false });
  });
  it("Close a job posting after staking on it", async () => {
    const jobPoster = await create_keypair();
    const mintAuthority = await create_keypair();
    const [job] = await get_pda_from_seeds([
      Buffer.from("boringlif"),
      jobPoster.publicKey.toBuffer(),
      Buffer.from(jobName),
    ]);
    await program.methods
      .addJob(jobName, jobDesc, jobLength, jobMetadataUri)
      .accounts({
        jobAddr: jobPoster.publicKey,
        job: job,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([jobPoster])
      .rpc({ skipPreflight: false });

    //creating spl token
    const mintAddress = await createMint(
      connection,
      mintAuthority,
      mintAuthority.publicKey,
      mintAuthority.publicKey,
      6
    );
    const jobPosterTokenAddress = await createAssociatedTokenAccount(
      connection,
      jobPoster,
      mintAddress,
      jobPoster.publicKey
    );
    const jobTokenAddress = await getAssociatedTokenAddress(
      mintAddress,
      job,
      true
    );
    await mintTo(
      connection,
      jobPoster,
      mintAddress,
      jobPosterTokenAddress,
      mintAuthority,
      mintAmount
    );
    await program.methods
      .stakeJob(stakeAmmount)
      .accounts({
        jobAddr: jobPoster.publicKey,
        job: job,
        jobAddrUsdcAccount: jobPosterTokenAddress,
        jobUsdcAccount: jobTokenAddress,
        usdcMint: mintAddress,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([jobPoster])
      .rpc({ skipPreflight: false });
    await program.methods
      .closeJob()
      .accounts({
        jobAddr: jobPoster.publicKey,
        jobAddrUsdcAccount: jobPosterTokenAddress,
        job: job,
        jobUsdcAccount: jobTokenAddress,
        usdcMint: mintAddress,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([jobPoster])
      .rpc({ skipPreflight: false });
  });
  it("Close a job posting without staking on it", async () => {
    const jobPoster = await create_keypair();
    const mintAuthority = await create_keypair();
    const [job] = await get_pda_from_seeds([
      Buffer.from("boringlif"),
      jobPoster.publicKey.toBuffer(),
      Buffer.from(jobName),
    ]);
    await program.methods
      .addJob(jobName, jobDesc, jobLength, jobMetadataUri)
      .accounts({
        jobAddr: jobPoster.publicKey,
        job: job,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([jobPoster])
      .rpc({ skipPreflight: false });

    //creating spl token
    const mintAddress = await createMint(
      connection,
      mintAuthority,
      mintAuthority.publicKey,
      mintAuthority.publicKey,
      6
    );
    await program.methods
      .closeJob()
      .accounts({
        jobAddr: jobPoster.publicKey,
        jobAddrUsdcAccount: null,
        job: job,
        jobUsdcAccount: null,
        usdcMint: mintAddress,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([jobPoster])
      .rpc({ skipPreflight: false });
  });
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

  it("Adds a verified freelancer", async () => {
    const [routerCreatorKeypair, nameRouterAccount] =
      await create_name_router();

    const [verifiedUserAccount] = await create_verified_user(
      routerCreatorKeypair,
      nameRouterAccount,
      userPubkey
    );
  });
});
