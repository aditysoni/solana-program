const { Connection, Keypair, SystemProgram, Transaction, LAMPORTS_PER_SOL, PublicKey } = require('@solana/web3.js');
const dotenv = require('dotenv');

dotenv.config();

async function transferSOL() {
    try {
        // Connect to the Solana devnet (use 'mainnet-beta' for production)
        const connection = new Connection('https://api.devnet.solana.com', 'confirmed');

        // Load sender's private key from environment variable
        const senderPrivateKey = process.env.SENDER_PRIVATE_KEY;
        if (!senderPrivateKey) {
            throw new Error('SENDER_PRIVATE_KEY not set in .env');
        }

        // Convert private key (base58 or byte array) to Keypair
        const senderKeypair = Keypair.fromSecretKey(
            Uint8Array.from(JSON.parse(senderPrivateKey)) // Adjust if your key is in base58
        );

        // Recipient's public key (replace with the actual recipient's address)
        const recipientPublicKey = new PublicKey('RECIPIENT_PUBLIC_KEY_HERE');

        // Amount to transfer (e.g., 0.1 SOL)
        const amount = 0.1 * LAMPORTS_PER_SOL; // SOL is measured in lamports (1 SOL = 1,000,000,000 lamports)

        // Create a transaction
        const transaction = new Transaction().add(
            SystemProgram.transfer({
                fromPubkey: senderKeypair.publicKey,
                toPubkey: recipientPublicKey,
                lamports: amount,
            })
        );

        // Estimate gas (optional, Solana uses lamports for fees)
        const { feeCalculator } = await connection.getRecentBlockhash();
        console.log(`Estimated transaction fee: ${feeCalculator.lamportsPerSignature} lamports`);

        // Sign and send the transaction
        const signature = await connection.sendTransaction(transaction, [senderKeypair]);
        console.log(`Transaction signature: ${signature}`);

        // Confirm the transaction
        const confirmation = await connection.confirmTransaction(signature, 'confirmed');
        if (confirmation.value.err) {
            throw new Error('Transaction failed');
        }

        console.log(`Successfully transferred ${amount / LAMPORTS_PER_SOL} SOL to ${recipientPublicKey.toBase58()}`);
    } catch (err) {
        console.error('Error transferring SOL:', err.message);
    }
}

transferSOL();