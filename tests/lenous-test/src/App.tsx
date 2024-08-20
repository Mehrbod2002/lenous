import { useMemo, useState, useEffect } from "react";
import {
  ConnectionProvider,
  WalletProvider,
  useAnchorWallet,
} from "@solana/wallet-adapter-react";
import {
  WalletModalProvider,
  WalletMultiButton,
} from "@solana/wallet-adapter-react-ui";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import IDL from "../../../target/idl/lenous.json";
import "@solana/wallet-adapter-react-ui/styles.css";
import { PhantomWalletAdapter } from "@solana/wallet-adapter-wallets";

const NETWORK = "http://127.0.0.1:8899";
const PROGRAM_ID = new PublicKey("Bims5KmWhFne1m1UT4bfSknBEoECeYfztoKrsR2jTnrA");

const createProgram = (provider: AnchorProvider): Program => {
  return new Program(IDL as any, PROGRAM_ID, provider);
};

const secretKey = Uint8Array.from([
  111, 52, 146, 191, 188, 82, 134, 162, 192, 82, 150, 45, 47, 229, 58, 233,
  41, 40, 21, 3, 173, 2, 202, 214, 162, 182, 75, 43, 46, 4, 106, 104, 117,
  216, 172, 53, 222, 70, 119, 60, 14, 58, 222, 26, 38, 127, 205, 174, 58,
  17, 214, 217, 254, 199, 161, 134, 60, 247, 131, 199, 40, 197, 97, 116
]);

const walletPayer = Keypair.fromSecretKey(secretKey);

function App() {
  const endpoint = useMemo(() => NETWORK, []);
  const connection = new Connection("http://127.0.0.1:8899", "confirmed");
  const [provider, setProvider] = useState<AnchorProvider | null>(null);
  const [program, setProgram] = useState<Program | null>(null);
  const wallet = useAnchorWallet();

  const wallets = useMemo(() => [
    new PhantomWalletAdapter(),
  ], []);

  useEffect(() => {
    if (wallet && connection) {
      const provider = new AnchorProvider(connection, wallet, {});
      setProvider(provider);
      setProgram(createProgram(provider));
    } else {
      setProvider(null);
      setProgram(null);
    }
  }, [wallet, connection]);

  const initializeUserAccount = async () => {
    if (provider && program) {
      try {
        const walletPublicKey = wallet?.publicKey;
        if (!walletPublicKey) {
          throw new Error("Wallet public key is not available.");
        }
        const [userAccountPDA] = await PublicKey.findProgramAddress(
          [Buffer.from("user_account"), walletPublicKey.toBuffer()],
          PROGRAM_ID
        );

        await program.methods.initializeUserAccount()
          .accounts({
            userAccount: userAccountPDA,
            systemProgram: SystemProgram.programId,
          })
          .rpc();

        console.log("User account initialized", userAccountPDA.toBase58());
      } catch (err) {
        console.error("Error initializing user account:", err);
      }
    }
  };
  console.log(wallet)
  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider autoConnect wallets={wallets}>
        <WalletModalProvider>
          <WalletMultiButton />
          <h1>Lenous</h1>
          <button
            onClick={initializeUserAccount}
            disabled={!provider}
          >
            Initialize User Account
          </button>
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}

export default App;
