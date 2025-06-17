import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ValidatorAnchorDemo } from "../target/types/validator_anchor_demo";
import { assert, expect } from "chai";

describe("validator_anchor_demo", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ValidatorAnchorDemo as Program<ValidatorAnchorDemo>;

  // Shared keypair across all tests
  const validator = anchor.web3.Keypair.generate();

  it("Creates a validator", async () => {
    await program.methods
      .createValidator(new anchor.BN(42), "KartikValidator")
      .accountsStrict({
        validator: validator.publicKey,
        user: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([validator])
      .rpc();

    const account = await program.account.validatorInfo.fetch(validator.publicKey);

    console.log("Validator created:", {
      id: account.id.toNumber(),
      name: account.name,
      isActive: account.isActive,
    });

    assert.strictEqual(account.id.toNumber(), 42);
    assert.strictEqual(account.name, "KartikValidator");
    assert.strictEqual(account.isActive, true);
  });

  it("Updates validator info", async () => {
    await program.methods
      .updateValidator("UpdatedValidator", false)
      .accountsStrict({
        validator: validator.publicKey,
      })
      .rpc();

    const updated = await program.account.validatorInfo.fetch(validator.publicKey);

    console.log("Validator updated:", {
      id: updated.id.toNumber(),
      name: updated.name,
      isActive: updated.isActive,
    });

    assert.strictEqual(updated.name, "UpdatedValidator");
    assert.strictEqual(updated.isActive, false);
  });

  it("Closes validator account", async () => {
    const preBalance = await provider.connection.getBalance(provider.wallet.publicKey);

    await program.methods
      .closeValidator()
      .accountsStrict({
        validator: validator.publicKey,
        refundTo: provider.wallet.publicKey,
      })
      .signers([]) // validator account was signed at init, but user is signer here
      .rpc();

    const postBalance = await provider.connection.getBalance(provider.wallet.publicKey);
    const lamportsRef = postBalance - preBalance;

    console.log("Refunded lamports from closed account:", lamportsRef);

    try {
      await program.account.validatorInfo.fetch(validator.publicKey);
      assert.fail("Account still exists after closure!");
    } catch (err:any) {
      expect(err.message).to.include("Account does not exist");
    }
  });
});
