import * as anchor from "@coral-xyz/anchor";
import { Connection, Keypair, clusterApiUrl, LAMPORTS_PER_SOL } from "@solana/web3.js";
import fs from "fs";

// Load your keypair from file
const walletKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("/Users/Office/solana-smartcontracts/counter-ts/myprogram-keypair.json", "utf-8")))
);

const connection = new Connection("https://api.devnet.solana.com", "confirmed");
const walletWrapper = new anchor.Wallet(walletKeypair);


  // Check new balance

const provider = new anchor.AnchorProvider(connection, walletWrapper, {
  preflightCommitment: "confirmed",
});

anchor.setProvider(provider);

// Load IDL and program ID
const idl = JSON.parse(fs.readFileSync("target/idl/counter_ts.json", "utf8"));
const programId = new anchor.web3.PublicKey("74QZ1uTUKCPsao19wAtRRxxQ441PeejhkAZBH7nw9EEN");

// Initialize the program with the correct constructor
const program = new anchor.Program(idl, provider);

(async () => {
  try {
    // Check wallet balance first
    const balance = await connection.getBalance(walletKeypair.publicKey);
    console.log(`Wallet balance: ${balance / LAMPORTS_PER_SOL} SOL`);
    
    if (balance < 0.01 * LAMPORTS_PER_SOL) {
      console.log("❌ Insufficient balance. Please airdrop SOL first.");
      console.log(`Run: solana airdrop 2 ${walletKeypair.publicKey.toBase58()} --url devnet`);
      return;
    }

    // Generate counter account
    const counterAccount = anchor.web3.Keypair.generate();
    console.log(`Creating counter account: ${counterAccount.publicKey.toBase58()}`);

    // Initialize the counter
    const tx = await program.methods
      .initialize()
      .accounts({
        counterAccount: counterAccount.publicKey,
        user: walletKeypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([counterAccount, walletKeypair])
      .rpc();

    console.log("✅ Counter account created successfully!");
    console.log(`Counter account: ${counterAccount.publicKey.toBase58()}`);
    console.log(`Transaction signature: ${tx}`);

  } catch (error) {
    console.error("❌ Deployment failed:");
    
    if (error.name === "SendTransactionError") {
      console.error("Transaction failed:", error.transactionMessage);
      
      // Try to get more detailed logs
      try {
        const logs = error.getLogs();
        console.error("Transaction logs:", logs);
      } catch (logError) {
        console.error("Could not retrieve logs:", logError);
      }
    } else {
      console.error(error);
    }
  }
})();