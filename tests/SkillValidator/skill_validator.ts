import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SkillValidator } from "../../target/types/skill_validator";

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
  const jobLength = { longTerm: {}};
  const jobName:string = "Solana dev";
  const jobDesc = "Build smart contracts";
  const jobMetadataUri = "https://github.com/defi-os/Issues";
  it("Create a job posting", async()=> {
    const jobPoster = await create_keypair();
    const [job] = await get_pda_from_seeds([
        Buffer.from("boringlif"),
        jobPoster.publicKey.toBuffer(),
        Buffer.from(jobName)
    ])
    await program.methods.addJob(jobName,jobDesc,jobLength,jobMetadataUri)
    .accounts({
        jobAddr: jobPoster.publicKey,
        job: job,
        systemProgram: web3.SystemProgram.programId
    }).signers([jobPoster])
    .rpc({skipPreflight: true});
  })
})