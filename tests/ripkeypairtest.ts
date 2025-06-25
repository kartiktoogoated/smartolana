// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { ValidatorAnchorDemo } from "../target/types/smartolana";
// import { assert, expect } from "chai";

// describe("smartolana", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.ValidatorAnchorDemo as Program<ValidatorAnchorDemo>;

//   // ✅ Shared keypair for validator account
//   const validator = anchor.web3.Keypair.generate();

//   it("Creates a validator", async () => {
//     await program.methods
//       .createValidator(new anchor.BN(42), "KartikValidator")
//       .accountsStrict({
//         validator: validator.publicKey,
//         user: provider.wallet.publicKey,
//         systemProgram: anchor.web3.SystemProgram.programId,
//       })
//       .signers([validator])
//       .rpc();

//     const account = await program.account.validatorInfo.fetch(validator.publicKey);

//     console.log("✅ Validator created:", {
//       pubkey: validator.publicKey.toBase58(),
//       id: account.id.toNumber(),
//       name: account.name,
//       isActive: account.isActive,
//     });

//     assert.strictEqual(account.id.toNumber(), 42);
//     assert.strictEqual(account.name, "KartikValidator");
//     assert.strictEqual(account.isActive, true);
//   });

//   it("Updates validator info", async () => {
//     await program.methods
//       .updateValidator("UpdatedValidator", false)
//       .accountsStrict({
//         validator: validator.publicKey,
//       })
//       .rpc();

//     const updated = await program.account.validatorInfo.fetch(validator.publicKey);

//     console.log("✅ Validator updated:", {
//       id: updated.id.toNumber(),
//       name: updated.name,
//       isActive: updated.isActive,
//     });

//     assert.strictEqual(updated.name, "UpdatedValidator");
//     assert.strictEqual(updated.isActive, false);
//   });

//   it("Closes validator account", async () => {
//     const preBalance = await provider.connection.getBalance(provider.wallet.publicKey);

//     await program.methods
//       .closeValidator()
//       .accountsStrict({
//         validator: validator.publicKey,
//         refundTo: provider.wallet.publicKey,
//       })
//       .rpc();

//     const postBalance = await provider.connection.getBalance(provider.wallet.publicKey);
//     const lamportsRef = postBalance - preBalance;

//     console.log("✅ Validator account closed. Lamports refunded:", lamportsRef);

//     try {
//       await program.account.validatorInfo.fetch(validator.publicKey);
//       assert.fail("Validator account still exists after closure!");
//     } catch (err: any) {
//       expect(err.message).to.include("Account does not exist");
//     }
//   });

//   // =====================================
//   // 🧪 Block 3 — Create PDA-Based Profile
//   // =====================================
//   it("Initializes a PDA profile for the user", async () => {
//     const [profilePda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
//       [Buffer.from("profile"), provider.wallet.publicKey.toBuffer()],
//       program.programId
//     );

//     await program.methods
//       .initProfile("Kartik")
//       .accountsStrict({
//         profile: profilePda,
//         authority: provider.wallet.publicKey,
//         systemProgram: anchor.web3.SystemProgram.programId,
//       })
//       .rpc();

//     const profileAccount = await program.account.userProfile.fetch(profilePda);

//     console.log("✅ PDA Profile created:", {
//       address: profilePda.toBase58(),
//       authority: profileAccount.authority.toBase58(),
//       name: profileAccount.name,
//       bump: profileAccount.bump,
//     });

//     assert.strictEqual(profileAccount.name, "Kartik");
//     assert.strictEqual(profileAccount.authority.toBase58(), provider.wallet.publicKey.toBase58());
//     assert.strictEqual(profileAccount.bump, bump);
//   });
// });
