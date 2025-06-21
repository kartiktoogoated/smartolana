# Solana Anchor Learning Journey

This repository tracks my end-to-end learning journey of building smart contracts (programs) on Solana using the Anchor framework. The repo is structured by **phases**, each adding new concepts and logic incrementally.

---

## 🧭 Overview

- ✅ **Network:** Solana Devnet
- ✅ **Framework:** [Anchor](https://book.anchor-lang.com/)
- ✅ **Frontend:** React + Wallet Adapter (Phantom)
- ✅ **Goal:** Learn to build PDA-based programs, token minting via CPI, and validator registration with ATAs.

---

## 📌 Phases

### 🔹 Phase 1: Basic Validator Initialization

- 🔑 Generate a validator using pubkey + private key
- 🧱 Created simple account storing `id`, `name`, `is_active`, and `authority`
- 🧪 Tested validator creation via CLI

### 🔹 Phase 2: Update and Delete Support

- 📝 Added `update_validator` and `close_validator` instructions
- 🔄 Used `has_one = authority` and `bump` for access control and reallocation
- ✅ Covered mutability, constraints, and authority checks

### 🔹 Phase 3: PDA-Based Profiles

- 🧬 Created a `UserProfile` account using a **PDA**:
  - `seeds = ["profile", authority]`
- 👤 Assigned authority + name to a profile
- 🔗 Linked validator creation to a profile

### 🔹 Phase 4: Multiple Validators per Profile

- 🌐 Enabled **many-to-one** mapping (multiple validators assigned to a single profile)
- 🛡️ Enforced ownership with `has_one = profile` and `has_one = authority`
- 🧠 Understood PDA constraints and key relationships

### 🔹 Phase 5: Token Minting via CPI

- 🪙 Created a **global mint** via `create_mint` instruction
- 🔑 Used a PDA mint authority: `seeds = ["mint-authority"]`
- 🧾 Minted tokens to validator’s **ATA** via CPI (`mint_to`)
- 🤝 Used `anchor_spl::token` and `associated_token`

---

## 🛠️ Frontend Summary

- 📦 Connected via `@solana/wallet-adapter-react` (Phantom)
- 🔘 Button to mint tokens using Anchor
- 🧩 Handles ATA generation and mint creation via client
- ⚠️ Type-safe with fallback alerts when wallet isn't connected

---

## 🚀 End Goal

- 🔧 Create a secure, decentralized **validator registry** with:
  - PDA-based ownership
  - Profile → multiple validators
  - Reward mechanism via token minting (Anchor CPI)
- 📚 Serve as a reference repo for anyone learning Solana development with Anchor

---

## 📁 Folder Structure

```bash
validator_anchor_demo/
├── programs/
│   └── validator_anchor_demo/   # Rust smart contract
├── target/                      # IDL + build output
├── app/                         # Frontend React app
│   └── src/
│       └── utils/               # Helper functions for minting, CPI
└── README.md