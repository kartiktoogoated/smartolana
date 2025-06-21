/* eslint-disable @typescript-eslint/no-unused-vars */
import {
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getAssociatedTokenAddress,
  } from "@solana/spl-token";
  import { getAnchorProvider } from "./provider";
  import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
  import * as anchor from "@coral-xyz/anchor";
  
  export const mintToValidator = async (
    wallet: anchor.Wallet,
    mintAddress: string,
    amount: number 
  ) => {
    const { provider, program } = getAnchorProvider(wallet);
  
    const mint = new PublicKey(mintAddress);
    const user = provider.wallet.publicKey;
  
    const validatorId = new anchor.BN(42);
  
    const [mintAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint-authority")],
      program.programId
    );
  
    const [validatorPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("validator"), user.toBuffer(), validatorId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
  
    const [profilePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("profile"), user.toBuffer()],
      program.programId
    );
  
    const ata = await getAssociatedTokenAddress(
      mint,
      user,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
  
    await program.methods
      .initValidator(validatorId, "Validator Node")
      .accountsStrict({
        validator: validatorPda,
        authority: user,
        profile: profilePda,
        validatorAta: ata,
        mint,
        mintAuthority,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .rpc();
  
    console.log("✅ Validator rewarded & initialized!");
  };
  