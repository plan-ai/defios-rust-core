import * as constant from "../constants";
import { rpcConfig } from "../test_config";
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Defios } from "../../target/types/defios";
import { getAssociatedTokenAddress } from "@solana/spl-token";
import * as ed from "@noble/ed25519";
// Configure the client to use the local cluster.
anchor.setProvider(anchor.AnchorProvider.env());

//testing defios workspace here
const program = anchor.workspace.Defios as Program<Defios>;
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
  return web3.PublicKey.findProgramAddressSync(seeds, program.programId);
}

async function get_metadata_account(mintKeypair) {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      constant.TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mintKeypair.toBuffer(),
    ],
    constant.TOKEN_METADATA_PROGRAM_ID
  )[0];
}

async function create_name_router() {
  //generating keypair and airdropping solana to it
  const routerCreatorKeypair = await create_keypair();

  //get public key of pda ideally generated using seeds
  const [nameRouterAccount] = await get_pda_from_seeds([
    Buffer.from(constant.signingName),
    Buffer.from(constant.signatureVersion.toString()),
    routerCreatorKeypair.publicKey.toBuffer(),
  ]);
  //call create name router function
  await program.methods
    .createNameRouter(constant.signingName, constant.signatureVersion)
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
    Buffer.from(`DefiOS(${constant.userName}, ${pubKey.toString()})`)
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
    Buffer.from(constant.userName),
    pubKey.toBuffer(),
    nameRouterAccount.toBuffer(),
  ]);
  //calls add verified user method
  await program.methods
    .addVerifiedUser(
      constant.userName,
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

async function create_spl_token(
  repositoryCreator,
  repositoryId = constant.repositoryId
) {
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

  return [
    repositoryAccount,
    repositoryCreatorTokenAccount,
    vestingTokenAccount,
    mint,
    vestingAccount,
  ];
}

export {
  create_keypair,
  create_name_router,
  create_spl_token,
  create_verified_user,
  get_pda_from_seeds,
  get_metadata_account,
};
