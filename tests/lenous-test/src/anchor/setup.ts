import { Program } from "@coral-xyz/anchor";
import IDL from "../../../../target/idl/lenous.json";
import { Connection, PublicKey } from "@solana/web3.js";

const programId = new PublicKey("92244zKyakcV9fa3kzH6AAWFDUwK5cHCueTZxaFgnXue");
const connection = new Connection("http://127.0.0.1:8899", "confirmed");

export const program = new Program<any>(IDL, programId, {
    connection,
});

export const [counterPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("counter")],
    program.programId,
);
