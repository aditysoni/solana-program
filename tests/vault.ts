import * as anchor from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
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

// Instantiate the program with provider and program ID
const program = new anchor.Program(idl, provider);

(async () => {
  // Generate a new keypair for the counter account
  const counterAccount = Keypair.generate();

  try {
    await program.methods
      .initialize()
      .accounts({
        counterAccount: counterAccount.publicKey,
        user: wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([counterAccount])
      .rpc();

    console.log("✅ Counter initialized at:", counterAccount.publicKey.toBase58());
  } catch (error) {
    console.error("❌ Failed to initialize counter:", error);
  }
})();
