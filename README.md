# Validator Anchor Token Demo - Solana PDA + Token Minting

This project demonstrates a full-featured Solana program using **Anchor** framework with PDA-based account architecture for managing:

* Token minting
* Validator registration
* Governance proposals and voting
* Token staking with time-locked vaults

---

## Features

* PDA-based Profile creation (UserProfile)
* Global Token Mint with PDA Mint Authority
* Validator metadata and token allocation
* Secure token transfers and burning
* Reassignable mint authority (for DAO control)
* Voting system (create, vote, prevent double-vote)
* StakeVault with time-lock and validator-linked staking

---

## Program Architecture

```
User Wallet
 └── init_profile         → Profile PDA ("profile", user)
 └── create_mint         → Global Mint PDA ("global-mint"), Mint Authority PDA ("mint-authority")
 └── init_validator      → Validator PDA ("validator", user, id), mints tokens
 └── transfer_tokens     → Direct token transfers between users
 └── burn_tokens         → Burns tokens from own ATA
 └── reassign_mint_auth  → Shifts authority to another pubkey
 └── create_proposal     → Governance proposal under profile
 └── vote_on_proposal    → Records vote by a validator
 └── stake_tokens        → Stake tokens via StakeVault PDA
```

---

## Key Accounts

| Account       | PDA? | Seeds                           | Purpose                                 |
| ------------- | ---- | ------------------------------- | --------------------------------------- |
| UserProfile   | Yes  | `["profile", user]`             | Stores user metadata and authority      |
| ValidatorInfo | Yes  | `["validator", user, id_bytes]` | Holds validator ID, status, profile ref |
| Mint          | Yes  | `["global-mint"]`               | SPL Token Mint                          |
| MintAuthority | Yes  | `["mint-authority"]`            | Used to mint/burn tokens securely       |
| StakeVault    | Yes  | `["stake-vault", user]`         | Tracks staked amount and timestamp      |
| TokenAccount  | No   | (ATA)                           | Holds actual token balances             |

---

## Instructions

### Profile & Mint

* `init_profile(name)` → Initializes user profile with bump.
* `create_mint()` → Creates mint account and PDA authority.

### Validators

* `init_validator(id, name)` → Registers validator, mints tokens.
* `update_validator(name, status)` → Allows update of metadata.
* `close_validator()` → Closes account and refunds lamports.

### Token Actions

* `transfer_tokens(amount)` → Token transfer between users.
* `burn_tokens(amount)` → Destroys user tokens.
* `reassign_mint_authority(pubkey)` → Updates mint authority.

### Governance

* `create_proposal(id, title, desc, deadline)` → Starts a vote.
* `vote_on_proposal(vote: bool)` → Records a single validator vote.

### Staking

* `stake_tokens(amount)` → Locks tokens in vault, tracks start time.

### Coming Next

* `unstake_tokens()` → Withdraws tokens after lock duration.
* `claim_reward()` → Calculates rewards over stake duration.
* `init_staking_pool()` → Adds pool config: reward/token/lock logic.

---

## Testing

* Anchor framework and Mocha tests
* Airdrops, ATA creation, CPIs fully tested
* Error conditions (expired vote, double-stake) validated

---

## Security Practices

* All PDAs have seed-based constraints
* Token transfers use CPIs
* Authority ownership validated in context
* Custom errors and logs ensure safety and transparency

---

## Built With

-   [Anchor](https://book.anchor-lang.com/)

-   [Solana](https://solana.com/)

-   [SPL Token](https://spl.solana.com/token)

-   Mocha + Chai (for test coverage)

