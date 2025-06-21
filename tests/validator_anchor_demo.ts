import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ValidatorAnchorDemo } from "../target/types/validator_anchor_demo";
import { assert, expect } from "chai";
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

describe("validator_anchor_demo", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ValidatorAnchorDemo as Program<ValidatorAnchorDemo>;

  const id = 42;
  const idBytes = new anchor.BN(id).toArrayLike(Buffer, "le", 8);

  const user = provider.wallet.publicKey;

  // PDA Definitions
  const [profilePda, profileBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("profile"), user.toBuffer()],
    program.programId
  );

  const [validatorPda, validatorBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("validator"), user.toBuffer(), idBytes],
    program.programId
  );

  const [mintPda, mintBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("global-mint")],
    program.programId
  );

  const [mintAuthPda, mintAuthBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("mint-authority")],
    program.programId
  );

  const validatorAta = getAssociatedTokenAddressSync(mintPda, user);

  it("Creates the mint", async () => {
    await program.methods
      .createMint()
      .accountsStrict({
        mint: mintPda,
        mintAuthority: mintAuthPda,
        payer: user,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    console.log("✅ Token mint created at:", mintPda.toBase58());
  });

  it("Initializes a PDA profile for the user", async () => {
    await program.methods
      .initProfile("Kartik")
      .accountsStrict({
        profile: profilePda,
        authority: user,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const profileAccount = await program.account.userProfile.fetch(profilePda);

    console.log("✅ PDA Profile created:", {
      address: profilePda.toBase58(),
      authority: profileAccount.authority.toBase58(),
      name: profileAccount.name,
      bump: profileAccount.bump,
    });

    assert.strictEqual(profileAccount.name, "Kartik");
    assert.strictEqual(profileAccount.authority.toBase58(), user.toBase58());
    assert.strictEqual(profileAccount.bump, profileBump);
  });

  it("Initializes a PDA validator and mints tokens to ATA", async () => {
    await program.methods
      .initValidator(new anchor.BN(id), "KartikValidator")
      .accountsStrict({
        validator: validatorPda,
        authority: user,
        profile: profilePda,
        validatorAta: validatorAta, // ✅ fixed here
        mint: mintPda,
        mintAuthority: mintAuthPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const account = await program.account.validatorInfo.fetch(validatorPda);

    console.log("✅ Validator PDA created:", {
      pubkey: validatorPda.toBase58(),
      id: account.id.toNumber(),
      name: account.name,
      isActive: account.isActive,
      authority: account.authority.toBase58(),
      profile: account.profile.toBase58(),
      bump: account.bump,
    });

    assert.strictEqual(account.id.toNumber(), id);
    assert.strictEqual(account.name, "KartikValidator");
    assert.strictEqual(account.isActive, true);
    assert.strictEqual(account.authority.toBase58(), user.toBase58());
    assert.strictEqual(account.profile.toBase58(), profilePda.toBase58());
    assert.strictEqual(account.bump, validatorBump);
  });

  it("Updates PDA validator info", async () => {
    await program.methods
      .updateValidator("UpdatedValidator", false)
      .accountsStrict({
        validator: validatorPda,
        authority: user,
        profile: profilePda,
      })
      .rpc();

    const updated = await program.account.validatorInfo.fetch(validatorPda);

    console.log("✅ Validator updated:", {
      id: updated.id.toNumber(),
      name: updated.name,
      isActive: updated.isActive,
    });

    assert.strictEqual(updated.name, "UpdatedValidator");
    assert.strictEqual(updated.isActive, false);
  });

  it("Closes PDA validator account", async () => {
    const preBalance = await provider.connection.getBalance(user);

    await program.methods
      .closeValidator()
      .accountsStrict({
        validator: validatorPda,
        authority: user,
        profile: profilePda,
      })
      .rpc();

    const postBalance = await provider.connection.getBalance(user);
    const lamportsRef = postBalance - preBalance;

    console.log("✅ Validator account closed. Lamports refunded:", lamportsRef);

    try {
      await program.account.validatorInfo.fetch(validatorPda);
      assert.fail("Validator account still exists after closure!");
    } catch (err: any) {
      expect(err.message).to.include("Account does not exist");
    }
  });
});
