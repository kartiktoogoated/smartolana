import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Smartolana } from "../target/types/smartolana";
import { assert, expect } from "chai";
import {
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccount,
  getAccount,
} from "@solana/spl-token";

describe("smartolana", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Smartolana as Program<Smartolana>;

  const id = 42;
  const idBytes = new anchor.BN(id).toArrayLike(Buffer, "le", 8);

  const user = provider.wallet.publicKey;

  const poolId = new anchor.BN(99);
  const poolIdBytes = poolId.toArrayLike(Buffer, "le", 8);

  const [stakingPoolPda, stakingPoolBump] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), user.toBuffer(), poolIdBytes],
      program.programId
    );

  // PDA Definitions
  const [profilePda, profileBump] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("profile"), user.toBuffer()],
      program.programId
    );

  const [validatorPda, validatorBump] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("validator"), user.toBuffer(), idBytes],
      program.programId
    );

  const [mintPda, mintBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("global-mint")],
    program.programId
  );

  const [mintAuthPda, mintAuthBump] =
    anchor.web3.PublicKey.findProgramAddressSync(
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
    console.log("🛠 Creating Profile PDA at:", profilePda.toBase58());

    await program.methods
      .initProfile("Kartik")
      .accountsStrict({
        profile: profilePda,
        authority: user,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const profileAccount = await program.account.userProfile.fetch(profilePda);

    console.log("✅ Profile Created:");
    console.log("• Address   :", profilePda.toBase58());
    console.log("• Authority :", profileAccount.authority.toBase58());
    console.log("• Name      :", profileAccount.name);
    console.log("• Bump      :", profileAccount.bump);

    assert.strictEqual(profileAccount.name, "Kartik");
    assert.strictEqual(profileAccount.authority.toBase58(), user.toBase58());
    assert.strictEqual(profileAccount.bump, profileBump);
  });

  it("Initializes a PDA validator and mints tokens to ATA", async () => {
    console.log("🛠 Creating Validator PDA at:", validatorPda.toBase58());
    console.log("🛠 Using Mint PDA:", mintPda.toBase58());
    console.log("🛠 Mint Auth PDA:", mintAuthPda.toBase58());
    console.log("🛠 Validator ATA:", validatorAta.toBase58());

    await program.methods
      .initValidator(new anchor.BN(id), "KartikValidator")
      .accountsStrict({
        validator: validatorPda,
        authority: user,
        profile: profilePda,
        validatorAta: validatorAta,
        mint: mintPda,
        mintAuthority: mintAuthPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const account = await program.account.validatorInfo.fetch(validatorPda);

    console.log("✅ Validator Created:");
    console.log("• Pubkey    :", validatorPda.toBase58());
    console.log("• ID        :", account.id.toNumber());
    console.log("• Name      :", account.name);
    console.log("• Active?   :", account.isActive);
    console.log("• Authority :", account.authority.toBase58());
    console.log("• Profile   :", account.profile.toBase58());
    console.log("• Bump      :", account.bump);

    assert.strictEqual(account.id.toNumber(), id);
    assert.strictEqual(account.name, "KartikValidator");
    assert.strictEqual(account.isActive, true);
    assert.strictEqual(account.authority.toBase58(), user.toBase58());
    assert.strictEqual(account.profile.toBase58(), profilePda.toBase58());
    assert.strictEqual(account.bump, validatorBump);
  });

  it("Transfers tokens to another user", async () => {
    const recipient = anchor.web3.Keypair.generate();
    const recipientAta = getAssociatedTokenAddressSync(
      mintPda,
      recipient.publicKey
    );

    // Airdrop lamports to recipient so it can create ATA
    await provider.connection.requestAirdrop(
      recipient.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await new Promise((r) => setTimeout(r, 2000)); // Wait for airdrop finality

    if (!provider.wallet || !provider.wallet.payer) {
      throw new Error("Wallet or payer not available");
    }

    // Create recipient ATA manually
    await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer, // // Signer that pays for the ATA creation
      mintPda,
      recipient.publicKey
    );

    console.log("🎯 Created ATA for recipient:", recipientAta.toBase58());

    const senderBalanceBefore = await getAccount(
      provider.connection,
      validatorAta
    );
    const recipientBalanceBefore = await getAccount(
      provider.connection,
      recipientAta
    );
    console.log(
      "💰 Sender balance before:",
      Number(senderBalanceBefore.amount)
    );
    console.log(
      "💰 Recipient balance before:",
      Number(recipientBalanceBefore.amount)
    );

    // Transfer 10 tokens (10_000_000_000 with 9 decimals)
    await program.methods
      .transferTokens(new anchor.BN(10_000_000_000))
      .accountsStrict({
        sender: user,
        from: validatorAta,
        to: recipientAta,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const senderBalanceAfter = await getAccount(
      provider.connection,
      validatorAta
    );
    const recipientBalanceAfter = await getAccount(
      provider.connection,
      recipientAta
    );
    console.log("✅ Token transfer complete!");
    console.log("💸 Sender balance after:", Number(senderBalanceAfter.amount));
    console.log(
      "🎉 Recipient balance after:",
      Number(recipientBalanceAfter.amount)
    );

    assert.strictEqual(
      Number(senderBalanceBefore.amount) - 10_000_000_000,
      Number(senderBalanceAfter.amount)
    );

    assert.strictEqual(
      Number(recipientBalanceAfter.amount),
      Number(recipientBalanceBefore.amount) + 10_000_000_000
    );
  });

  it("Burns tokens from user's ATA", async () => {
    const burnAmount = new anchor.BN(5_000_000_000); // Burn 5 tokens (9 decimals)

    const balanceBefore = await getAccount(provider.connection, validatorAta);

    await program.methods
      .burnTokens(burnAmount)
      .accountsStrict({
        owner: user,
        ownerAta: validatorAta,
        mint: mintPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const balanceAfter = await getAccount(provider.connection, validatorAta);

    console.log("🔥 Burned tokens!");
    console.log("💰 Balance before:", Number(balanceBefore.amount));
    console.log("💸 Balance after :", Number(balanceAfter.amount));

    assert.strictEqual(
      Number(balanceBefore.amount) - burnAmount.toNumber(),
      Number(balanceAfter.amount)
    );
  });

  it("Initializes a staking pool", async () => {
    await program.methods
      .initStakingPool(poolId, "Default Pool", new anchor.BN(1000000)) // 0.001 per sec
      .accountsStrict({
        pool: stakingPoolPda,
        authority: user,
        stakeMint: mintPda,
        rewardMint: mintPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const poolAccount = await program.account.stakingPool.fetch(stakingPoolPda);

    console.log("✅ StakingPool Created:");
    console.log("• ID      :", poolAccount.id.toNumber());
    console.log("• Name    :", poolAccount.name);
    console.log("• Mint    :", poolAccount.stakeMint.toBase58());
    console.log("• Reward/s:", poolAccount.rewardPerSecond.toString());

    expect(poolAccount.id.toNumber()).to.equal(poolId.toNumber());
    expect(poolAccount.name).to.equal("Default Pool");
    expect(poolAccount.stakeMint.toBase58()).to.equal(mintPda.toBase58());
    expect(poolAccount.rewardPerSecond.toNumber()).to.equal(1000000);
  });

  it("Stakes tokens from user ATA to stake vault", async () => {
    const stakeAmount = new anchor.BN(5_000_000_000); // 5 tokens (assuming 9 decimals)

    // Derive stake vault PDA
    const [stakeVaultPda, stakeVaultBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("stake-vault"), user.toBuffer()],
        program.programId
      );

    // Derive user ATA and stake vault ATA
    const userAta = getAssociatedTokenAddressSync(mintPda, user);
    const vaultAta = getAssociatedTokenAddressSync(
      mintPda,
      stakeVaultPda,
      true // ✅ allowOwnerOffCurve
    );

    // Fetch balances before staking
    const userBefore = await getAccount(provider.connection, userAta);
    const vaultBefore = await getAccount(provider.connection, vaultAta).catch(
      () => ({
        amount: BigInt(0),
      })
    );

    // Call the stake_tokens instruction
    await program.methods
      .stakeTokens(stakeAmount)
      .accountsStrict({
        user,
        profile: profilePda,
        stakeVault: stakeVaultPda,
        pool: stakingPoolPda,
        userAta,
        vaultAta,
        stakeMint: mintPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    // Fetch updated stakeVault account
    const stakeVault = await program.account.stakeVault.fetch(stakeVaultPda);
    const userAfter = await getAccount(provider.connection, userAta);
    const vaultAfter = await getAccount(provider.connection, vaultAta);

    // Logs
    console.log("✅ Staked successfully:");
    console.log("• Vault Owner      :", stakeVault.owner.toBase58());
    console.log("• Stake Amount     :", stakeVault.amount.toString());
    console.log("• Start Stake Time :", stakeVault.startStakeTime.toString());

    // Assertions
    assert.strictEqual(stakeVault.owner.toBase58(), user.toBase58());
    assert.strictEqual(stakeVault.profile.toBase58(), profilePda.toBase58());
    assert.strictEqual(stakeVault.amount.toString(), stakeAmount.toString());

    assert.strictEqual(
      Number(userBefore.amount) - stakeAmount.toNumber(),
      Number(userAfter.amount)
    );

    assert.strictEqual(
      Number(vaultBefore.amount) + stakeAmount.toNumber(),
      Number(vaultAfter.amount)
    );
  });

  it("Claims rewards after staking duration", async () => {
    const [stakeVaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("stake-vault"), user.toBuffer()],
      program.programId
    );

    const userRewardAta = getAssociatedTokenAddressSync(mintPda, user);
    const [mintAuthPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("mint-authority")],
      program.programId
    );

    console.log("⏳ Waiting to accumulate reward...");
    await new Promise((res) => setTimeout(res, 3000)); // simulate time delay

    const before = await getAccount(provider.connection, userRewardAta);

    await program.methods
      .claimReward()
      .accountsStrict({
        user,
        stakeVault: stakeVaultPda,
        pool: stakingPoolPda,
        userRewardAta,
        rewardMint: mintPda,
        mintAuthority: mintAuthPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const after = await getAccount(provider.connection, userRewardAta);
    const claimed = Number(after.amount) - Number(before.amount);

    console.log("✅ Reward claimed:", claimed);
    assert.ok(claimed > 0, "Reward should be greater than 0");
  });

  it("Unstakes tokens after lock period", async () => {
    const stakeAmount = new anchor.BN(5_000_000_000); // 5 tokens
    const [stakeVaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("stake-vault"), user.toBuffer()],
      program.programId
    );

    const userAta = getAssociatedTokenAddressSync(mintPda, user);
    const vaultAta = getAssociatedTokenAddressSync(
      mintPda,
      stakeVaultPda,
      true
    );

    const vaultBefore = await getAccount(provider.connection, vaultAta);
    const userBefore = await getAccount(provider.connection, userAta);
    const stakeVault = await program.account.stakeVault.fetch(stakeVaultPda);

    const nowBn = new anchor.BN(Math.floor(Date.now() / 1000));
    const lockPeriod = new anchor.BN(3600);
    const unlockTime = stakeVault.startStakeTime.add(lockPeriod);

    // Simulate delay if needed (use shorter lock in local/test env)
    if (nowBn.lt(unlockTime)) {
      console.log("⏱ Waiting (simulated)... stake is still locked.");
      await new Promise((res) => setTimeout(res, 3000));
    }

    await program.methods
      .unstakeTokens()
      .accountsStrict({
        user,
        stakeVault: stakeVaultPda,
        userAta,
        vaultAta,
        stakeMint: mintPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const vaultAfter = await getAccount(provider.connection, vaultAta);
    const userAfter = await getAccount(provider.connection, userAta);
    const updatedVault = await program.account.stakeVault.fetch(stakeVaultPda);

    console.log("✅ Unstaked tokens from vault:");
    console.log(
      "• Returned to User:",
      Number(userAfter.amount) - Number(userBefore.amount)
    );
    console.log("• Vault now holds :", Number(vaultAfter.amount));

    assert.strictEqual(Number(vaultAfter.amount), 0);
    assert.strictEqual(updatedVault.amount.toNumber(), 0);
    assert.strictEqual(updatedVault.startStakeTime.toNumber(), 0);
  });

  it("Reassigns the mint authority", async () => {
    const newAuthority = anchor.web3.Keypair.generate();

    // Airdrop some SOL to the new authority (for future txns if needed)
    await provider.connection.requestAirdrop(
      newAuthority.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await new Promise((r) => setTimeout(r, 2000)); // wait for finality

    // Call the reassign_mint_authority instruction
    await program.methods
      .reassignMintAuthority(newAuthority.publicKey)
      .accountsStrict({
        mint: mintPda,
        mintAuthority: mintAuthPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // Get the parsed account info for the mint
    const mintInfo = await provider.connection.getParsedAccountInfo(mintPda);

    if (!mintInfo.value) throw new Error("Mint account not found");

    // Check if data is parsed (not raw Buffer)
    if ("parsed" in mintInfo.value.data) {
      const parsed = mintInfo.value.data.parsed;

      const actualAuthority = parsed?.info?.mintAuthority;
      console.log(
        "🔁 Reassigned Mint Authority to:",
        newAuthority.publicKey.toBase58()
      );
      console.log("🧾 On-chain Mint Authority     :", actualAuthority);

      expect(actualAuthority).to.equal(newAuthority.publicKey.toBase58());
    } else {
      throw new Error("Account data is not parsed");
    }
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

  it("Creates a proposal using PDA", async () => {
    const proposalId = new anchor.BN(1);
    const deadline = Math.floor(Date.now() / 1000) + 3600; // 1 hour from now

    const [proposalPda, proposalBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("proposal"),
          profilePda.toBuffer(),
          proposalId.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

    console.log("📝 Proposal PDA:", proposalPda.toBase58());

    await program.methods
      .createProposal(
        new anchor.BN(proposalId), // proposal_id
        "Decentralize Mint Access", // title
        "Proposal to allow multiple mint signers", // description
        new anchor.BN(deadline)
      )
      .accountsStrict({
        profile: profilePda,
        proposal: proposalPda,
        authority: user,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const proposalAccount = await program.account.proposal.fetch(proposalPda);

    console.log("✅ Proposal Created:");
    console.log("• Title       :", proposalAccount.title);
    console.log("• Description :", proposalAccount.description);
    console.log("• Created At  :", proposalAccount.createdAt.toString());
    console.log("• Deadline    :", proposalAccount.deadline.toString());

    assert.strictEqual(proposalAccount.title, "Decentralize Mint Access");
    assert.strictEqual(
      proposalAccount.description,
      "Proposal to allow multiple mint signers"
    );
    assert.strictEqual(
      proposalAccount.profile.toBase58(),
      profilePda.toBase58()
    );
    assert.strictEqual(proposalAccount.bump, proposalBump);
    assert.strictEqual(proposalAccount.id.toNumber(), Number(proposalId));
  });

  it("Allows a valid vote from validator", async () => {
    const proposalId = new anchor.BN(2);
    const deadline = Math.floor(Date.now() / 1000) + 3600; // 1 hour later

    const [proposalPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("proposal"),
        profilePda.toBuffer(),
        proposalId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    const [votePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vote"), proposalPda.toBuffer(), validatorPda.toBuffer()],
      program.programId
    );

    // Create a new proposal
    await program.methods
      .createProposal(
        proposalId,
        "Enable Logging",
        "Add validator event logging",
        new anchor.BN(deadline)
      )
      .accountsStrict({
        profile: profilePda,
        proposal: proposalPda,
        authority: user,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Vote on it
    await program.methods
      .voteOnProposal(true)
      .accountsStrict({
        authority: user,
        profile: profilePda,
        validator: validatorPda,
        proposal: proposalPda,
        voteRecord: votePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const voteAccount = await program.account.voteRecord.fetch(votePda);

    console.log("🗳 Vote Cast:", {
      vote: voteAccount.vote,
      timestamp: voteAccount.timestamp.toString(),
    });

    assert.strictEqual(voteAccount.vote, true);
    assert.strictEqual(
      voteAccount.validator.toBase58(),
      validatorPda.toBase58()
    );
    assert.strictEqual(voteAccount.proposal.toBase58(), proposalPda.toBase58());
  });

  it("Rejects duplicate vote from same validator", async () => {
    const proposalId = new anchor.BN(3);
    const deadline = Math.floor(Date.now() / 1000) + 3600;

    const [proposalPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("proposal"),
        profilePda.toBuffer(),
        proposalId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    const [votePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vote"), proposalPda.toBuffer(), validatorPda.toBuffer()],
      program.programId
    );

    await program.methods
      .createProposal(
        proposalId,
        "Add Alerting",
        "Notify on critical state",
        new anchor.BN(deadline)
      )
      .accountsStrict({
        profile: profilePda,
        proposal: proposalPda,
        authority: user,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    await program.methods
      .voteOnProposal(false)
      .accountsStrict({
        authority: user,
        profile: profilePda,
        validator: validatorPda,
        proposal: proposalPda,
        voteRecord: votePda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    try {
      await program.methods
        .voteOnProposal(true)
        .accountsStrict({
          authority: user,
          profile: profilePda,
          validator: validatorPda,
          proposal: proposalPda,
          voteRecord: votePda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail("Should not allow double voting");
    } catch (err: any) {
      console.log("✅ Rejected duplicate vote:", err.message);
      expect(err.message).to.include("already in use");
    }
  });

  it("Rejects vote on expired proposal", async () => {
    const proposalId = new anchor.BN(999);
    const deadline = Math.floor(Date.now() / 1000) + 2; // 2 seconds from now

    const [proposalPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("proposal"),
        profilePda.toBuffer(),
        proposalId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    const [votePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vote"), proposalPda.toBuffer(), validatorPda.toBuffer()],
      program.programId
    );

    // Create the proposal with a valid short deadline
    await program.methods
      .createProposal(
        proposalId,
        "Short-lived Proposal",
        "Expires fast",
        new anchor.BN(deadline)
      )
      .accountsStrict({
        profile: profilePda,
        proposal: proposalPda,
        authority: user,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Wait for the deadline to expire
    await new Promise((res) => setTimeout(res, 3000));

    // Now try voting — should fail due to deadline
    try {
      await program.methods
        .voteOnProposal(true)
        .accountsStrict({
          authority: user,
          profile: profilePda,
          validator: validatorPda,
          proposal: proposalPda,
          voteRecord: votePda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();
      assert.fail("Vote on expired proposal should have failed");
    } catch (err: any) {
      console.log("✅ Rejected expired vote:", err.message);
      expect(err.message).to.include("ProposalExpired");
    }
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
