import * as anchor from "@coral-xyz/anchor";
import {
  Connection,
  Keypair,
  clusterApiUrl,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  Transaction,
} from "@solana/web3.js";

import fs from "fs";

// Load your keypair from file
const secret = JSON.parse(
  fs.readFileSync("/Users/Office/.config/solana/id.json", "utf8")
);
const walletKeypair = Keypair.fromSecretKey(Uint8Array.from(secret));

// Create Anchor Provider
const connection = new Connection("https://api.devnet.solana.com", "confirmed");
const wallet = new anchor.Wallet(walletKeypair);
const provider = new anchor.AnchorProvider(connection, wallet, {
  preflightCommitment: "confirmed",
});
anchor.setProvider(provider);

// Load IDL and Program ID
const idl = JSON.parse(fs.readFileSync("target/idl/counter_ts.json", "utf8"));
const programId = new PublicKey("74QZ1uTUKCPsao19wAtRRxxQ441PeejhkAZBH7nw9EEN");

const program = new anchor.Program(idl, provider);

(async () => {
  const counterAccount = Keypair.generate();

  // Priority fee: add Compute Budget instructions
  const computeBudgetIx1 = anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
    units: 200_000, // you can tweak this based on your compute
  });

  const computeBudgetIx2 = anchor.web3.ComputeBudgetProgram.setComputeUnitPrice({
    microLamports: 5000, // higher = faster inclusion, 5000 = 0.005 SOL per CU
  });

  const tx = await program.methods
    .initialize()
    .accounts({
      counterAccount: counterAccount.publicKey,
      user: wallet.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([counterAccount])
    .preInstructions([computeBudgetIx1, computeBudgetIx2]) // ⬅️ Priority fee instructions
    .rpc();

  console.log("✅ Counter initialized with priority fees at:", counterAccount.publicKey.toBase58());
})();
