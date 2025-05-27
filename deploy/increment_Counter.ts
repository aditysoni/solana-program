import * as anchor from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import fs from "fs";

// Load your keypair from file
const walletKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("/Users/Office/solana-smartcontracts/counter-ts/myprogram-keypair.json", "utf-8")))
);

const connection = new Connection("https://api.devnet.solana.com", "confirmed");
const walletWrapper = new anchor.Wallet(walletKeypair);

const provider = new anchor.AnchorProvider(connection, walletWrapper, {
  preflightCommitment: "confirmed",
});

anchor.setProvider(provider);

// Load IDL and program ID
const idl = JSON.parse(fs.readFileSync("target/idl/counter_ts.json", "utf8"));
const programId = new PublicKey("74QZ1uTUKCPsao19wAtRRxxQ441PeejhkAZBH7nw9EEN");

// Initialize the program
const program : any = new anchor.Program(idl, provider);

(async () => {
  // Replace with the actual public key of the initialized counter_account
  const counterAccountPubkey = new PublicKey("2zxdFmaPADwfkmToAomdortPZVhvwjLVJtkcGupVEbZx");

  try {
    // Call the incerment function (note the typo in the IDL)
    await program.methods
      .incerment() // Use 'incerment' due to typo in IDL
      .accounts({
        counterAccount: counterAccountPubkey,
      })
      .rpc();

    console.log("Counter incremented successfully!");
    console.log("Counter account:", counterAccountPubkey.toBase58());

    // Fetch and display the updated counter value
    const counterAccount = await program.account.counter.fetch(counterAccountPubkey); // Use capital 'C' for Counter
    console.log("Current counter value:", counterAccount.count.toString());
  } catch (error) {
    console.error("Error incrementing or fetching counter:", error);
  }
})();