import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CounterTs } from "../target/types/counter_ts";

const main = async () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CounterTs as Program<CounterTs>;

  const counterAccount = anchor.web3.Keypair.generate();

  // Initialize the counter
  await program.methods
    .initialize()
    .accounts({
      counterAccount: counterAccount.publicKey,
      user: provider.wallet.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([counterAccount])
    .rpc();

  console.log("Counter initialized at:", counterAccount.publicKey.toBase58());
};

main();
