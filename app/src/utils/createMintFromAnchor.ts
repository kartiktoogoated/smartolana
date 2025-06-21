/* eslint-disable @typescript-eslint/no-explicit-any */
import { getAnchorProvider } from "./provider";
import {
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";

export const createMintFromAnchor = async (wallet: anchor.Wallet): Promise<string> => {
  const { program } = getAnchorProvider(wallet);

  const [mintPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("global-mint")],
    program.programId
  );

  const [mintAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from("mint-authority")],
    program.programId
  );

  await program.methods
    .createMint()
    .accounts({
      mint: mintPda,
      mintAuthority: mintAuthority,
      payer: wallet.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
    } as any)
    .rpc();

  return mintPda.toBase58();
}; 