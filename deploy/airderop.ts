import * as anchor from "@coral-xyz/anchor";
import { Connection, Keypair, clusterApiUrl, LAMPORTS_PER_SOL } from "@solana/web3.js";
import fs from "fs";

// Load your keypair from file
const walletKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("/Users/Office/solana-smartcontracts/counter-ts/myprogram-keypair.json", "utf-8")))
);

const connection = new Connection("https://api.devnet.solana.com", "confirmed");

(async () => {
  try {
    console.log(`Requesting airdrop for: ${walletKeypair.publicKey.toBase58()}`);
    
    // Request 2 SOL airdrop
    const signature = await connection.requestAirdrop(
      walletKeypair.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    
    // Wait for confirmation
    await connection.confirmTransaction(signature);
    console.log(`✅ Airdrop successful! Transaction signature: ${signature}`);
    
    // Check new balance
    const balance = await connection.getBalance(walletKeypair.publicKey);
    console.log(`New balance: ${balance / LAMPORTS_PER_SOL} SOL`);
    
  } catch (error) {
    console.error("❌ Airdrop failed:", error);
    console.log("Try using the Solana CLI: solana airdrop 2 --url devnet");
  }
})();