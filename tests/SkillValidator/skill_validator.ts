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
});
