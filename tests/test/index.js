import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import { readFileSync } from 'fs';
import { resolve } from 'path';
import fs from 'fs';
import path from 'path';
import { AnchorProvider, Program, Wallet } from '@coral-xyz/anchor';

const loadWallet = (filePath) => {
    const secretKeyArray = JSON.parse(fs.readFileSync(filePath, 'utf8'));
    const secretKey = new Uint8Array(secretKeyArray);
    return Keypair.fromSecretKey(secretKey);
};

const idlPath = resolve('./../../target/idl/lenous.json');
const idl = JSON.parse(readFileSync(idlPath, 'utf8'));
const programID = new PublicKey('F9GEX6D75oQA5h1Scn4WgH7zFxCkeVCn2HMSnrbhfH6Z');
const connection = new Connection('http://127.0.0.1:8899', 'confirmed');

const walletPaths = [
    path.resolve('./wallets/GpFiuAPkZAtRNp8u8ncVEEQW57S9BcvaVd7C1Zm7QwD_secret_key.json'),
    path.resolve('./wallets/7kx8AiECyyxxSDHR42HdB6HWXeqPvHgygvt5TGfSyvLV_secret_key.json'),
    path.resolve('./wallets/5hwBXqoZZ1XesFjPUT6scfPdNvC8V7oaXhmf9mBC3wm8_secret_key.json'),
];

const keypair = loadWallet(walletPaths[0]);
const wallet = new Wallet(keypair);
const provider = new AnchorProvider(connection, wallet, AnchorProvider.defaultOptions());
const program = new Program(idl, programID);

async function callInitialize() {
    console.log("called")
    // const accounts = {
    //     admin: wallet.publicKey,
    //     dexAccount: new PublicKey('YourDexAccountPublicKeyHere'),
    //     usdtTokenAccount: new PublicKey('YourUSDTTokenAccountPublicKeyHere'),
    //     usdcMint: new PublicKey('YourUSDCMintPublicKeyHere'),
    //     systemProgram: new PublicKey('11111111111111111111111111111111'),
    //     tokenProgram: new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA')
    // };

    // const usdtMint = new PublicKey('YourUSDTMintPublicKeyHere');
    // const usdcMint = new PublicKey('YourUSDCMintPublicKeyHere');
    // const feeRate = 1000; 

    try {
        const tx = await program.methods();
        console.log(tx)
        console.log("Transaction successful:", tx);
    } catch (error) {
        console.error("Error calling method:", error);
    }
}

callInitialize().catch(console.error);
