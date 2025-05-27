import * as anchor from "@coral-xyz/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
  ComputeBudgetProgram,
  Transaction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";
import fs from "fs";

// Load your keypair from file
const walletKeypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("/Users/Office/solana-smartcontracts/counter-ts/myprogram-keypair.json", "utf-8")))
);

const connection : any = new Connection("https://api.devnet.solana.com", "confirmed");
const walletWrapper = new anchor.Wallet(walletKeypair);
const provider = new anchor.AnchorProvider(connection, walletWrapper, {
  preflightCommitment: "confirmed",
});
anchor.setProvider(provider);

// Load IDL and program ID
let idl;
try {
  idl = JSON.parse(fs.readFileSync("/Users/Office/solana-smartcontracts/counter-ts/target/idl/counter_ts.json", "utf-8"));
  console.log("IDL accounts:", idl.accounts); // Log accounts section
} catch (error) {
  console.error("Error loading IDL:", error);
  process.exit(1);
}

const programId = new PublicKey("74QZ1uTUKCPsao19wAtRRxxQ441PeejhkAZBH7nw9EEN");
const program :any = new anchor.Program(idl, provider);

// Log available accounts
console.log("Available accounts in program.account:", Object.keys(program.account));

(async () => {
  const counterAccountPubkey = new PublicKey("2zxdFmaPADwfkmToAomdortPZVhvwjLVJtkcGupVEbZx");

  try {
    // Build the increment instruction
    const instruction = await program.methods
      .incerment() // Use 'incerment' due to typo in IDL
      .accounts({
        counterAccount: counterAccountPubkey,
      })
      .instruction();

    // Get the latest blockhash
    const { blockhash } = await connection.getLatestBlockhash("confirmed");

    // Create a TransactionMessage
    const message = new TransactionMessage({
      payerKey: walletKeypair.publicKey,
      recentBlockhash: blockhash,
      instructions: [
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 }), // Max CU for simulation
        instruction,
      ],
    }).compileToV0Message();

    // Create a VersionedTransaction
    const versionedTransaction = new VersionedTransaction(message);

    // Simulate the transaction
    const simulation = await connection.simulateTransaction(versionedTransaction, {
      sigVerify: false,
      commitment: "confirmed",
      accounts: {
        encoding: "base64",
        addresses: [counterAccountPubkey], // Include counter account data
      },
    });

    if (simulation.value.err) {
      console.error("Simulation failed:", simulation.value.err);
      console.error("Simulation logs:", simulation.value.logs);
      return;
    }

    console.log("Estimated compute units consumed:", simulation.value.unitsConsumed);
    console.log("Simulation logs:", simulation.value.logs);
    console.log("Simulated account data:", simulation.value.accounts);

    // Calculate transaction fee (5 lamports per signature)
    const signatures = 1; // Only walletKeypair signs
    const feeInLamports = signatures * 5_000; // 0.000005 SOL per signature
    console.log("Estimated transaction fee:", feeInLamports / 1_000_000_000, "SOL");

    // Execute the transaction
    const signature = await program.methods
      .incerment()
      .accounts({
        counterAccount: counterAccountPubkey,
      })
      .signers([walletKeypair])
      .rpc();
    console.log("Counter incremented successfully! Signature:", signature);

    // Fetch and display the updated counter value
    const counterAccount = await program.account.counter.fetch(counterAccountPubkey);
    console.log("Current counter value:", counterAccount.count.toString());
  } catch (error) {
    console.error("Error incrementing or fetching counter:", error);
    if (error.logs) {
      console.error("Transaction logs:", error.logs);
    }
  }
})();