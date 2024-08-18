import { Keypair } from '@solana/web3.js';
import fs from 'fs';
import path from 'path';

// Function to create and save a new wallet
const createWallet = () => {
    const keypair = Keypair.generate();
    const secretKey = keypair.secretKey;
    const publicKey = keypair.publicKey.toBase58();
    const secretKeyPath = path.resolve(`./wallets/${publicKey}_secret_key.json`);

    // Create the wallets directory if it doesn't exist
    if (!fs.existsSync(path.resolve('./wallets'))) {
        fs.mkdirSync(path.resolve('./wallets'));
    }

    // Save the secret key to a file
    fs.writeFileSync(secretKeyPath, JSON.stringify(Array.from(secretKey)));

    return {
        publicKey,
        secretKeyPath,
    };
};

// Generate and save three sample wallets
for (let i = 0; i < 3; i++) {
    const wallet = createWallet();
    console.log(`Wallet ${i + 1}:`);
    console.log(`Public Key: ${wallet.publicKey}`);
    console.log(`Secret Key Path: ${wallet.secretKeyPath}`);
    console.log('----------------------------------------');
}
