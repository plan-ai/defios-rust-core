import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Defios } from '../target/types/defios';
import { Metaplex, keypairIdentity, bundlrStorage, compareAmounts } from "@metaplex-foundation/js";
import { TokenStandard, Metadata, PROGRAM_ADDRESS } from "@metaplex-foundation/mpl-token-metadata";
import * as mpl from "@metaplex-foundation/mpl-token-metadata";
import * as ed from '@noble/ed25519';
import { PublicKey } from '@saberhq/solana-contrib';
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    createAssociatedTokenAccount,
    createAssociatedTokenAccountInstruction,
    createInitializeMintInstruction,
    createMintToCheckedInstruction,
    getAssociatedTokenAddress,
    MintLayout,
    TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import { Connection, clusterApiUrl, Keypair, TransactionMessage, TransactionInstruction, VersionedTransaction } from "@solana/web3.js";
import { readFileSync } from "fs"
import sha1 from 'sha1';
import { bs58 } from '@project-serum/anchor/dist/cjs/utils/bytes';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';

describe('defios', async () => {

    const connection = new Connection(clusterApiUrl("devnet"));
    const wallet = Keypair.fromSecretKey(
        Uint8Array.from([183, 17, 185, 36, 109, 137, 119, 166, 151, 221, 22, 157, 213, 239, 184, 72, 165, 20, 132, 67, 193, 94, 137, 94, 37, 41, 156, 148, 58, 103, 129, 170, 50, 29, 169, 61, 98, 171, 169, 99, 50, 188, 34, 236, 197, 52, 74, 184, 177, 137, 205, 151, 169, 3, 236, 213, 193, 129, 38, 138, 170, 184, 131, 102])
    );
    const { web3 } = anchor;
    const metaplex = Metaplex.make(connection)
        .use(keypairIdentity(wallet))
        .use(bundlrStorage());


    it('Creates Token', async () => {
        try {
            const mint = await metaplex.tokens().createMint({
                mintAuthority: wallet.publicKey,
                decimals: 6
            })
            console.log(mint)
            const mintKeypair = mint.mintSigner
            const metadataPDA = metaplex.nfts().pdas().metadata({
                mint: mintKeypair.publicKey,
            })
            // const token = await metaplex.tokens().createToken({
            //     mint: mintKeypair.publicKey,
            //     owner: wallet.publicKey,
            // })
            // console.log(token)

            const accounts = {
                metadata: metadataPDA,
                mintKeypair,
                mintAuthority: wallet.publicKey,
                payer: wallet.publicKey,
                updateAuthority: wallet.publicKey,
            }
            const dataV2 = {
                name: "Fake USD Token",
                symbol: "FUD",
                uri: "https://shdw-drive.genesysgo.net/ArP7jjhVZsp7vkzteU7mpKA1fyHRhv4ZBz6gR7MJ1JTC/metadata.json",
                // we don't need that
                sellerFeeBasisPoints: 0,
                creators: null,
                collection: null,
                uses: null
            }
            let ix;
            const args = {
                createMetadataAccountArgsV2: {
                    data: dataV2,
                    isMutable: true
                }
            };

            ix = mpl.createCreateMetadataAccountV2Instruction({
                metadata: accounts.metadata,
                mint: accounts.mintKeypair.publicKey,
                mintAuthority: accounts.mintAuthority,
                payer: accounts.payer,
                updateAuthority: accounts.updateAuthority,
            }, args);

            const instructions: TransactionInstruction[] = [
                ix
            ];
            let latestBlockhash = await connection.getLatestBlockhash();
            const messageV0 = new TransactionMessage({
                payerKey: wallet.publicKey,
                recentBlockhash: latestBlockhash.blockhash,
                instructions: instructions
            }).compileToV0Message();
            console.log("   ✅ - Compiled Transaction Message");

            const transaction = new VersionedTransaction(messageV0);
            transaction.sign([wallet]);
            console.log("   ✅ - Transaction Signed");
            const txid = await connection.sendTransaction(transaction, { maxRetries: 5 });
            console.log("   ✅ - Transaction sent to network");
            const confirmation = await connection.confirmTransaction({
                signature: txid,
                blockhash: latestBlockhash.blockhash,
                lastValidBlockHeight: latestBlockhash.lastValidBlockHeight
            })
            if (confirmation.value.err) { throw new Error("   ❌ - Transaction not confirmed.") }
            console.log('🎉 Transaction Succesfully Confirmed!', '\n', `https://explorer.solana.com/tx/${txid}?cluster=devnet`);
        }
        catch (e) {
            console.log(e)
        }
    })
});