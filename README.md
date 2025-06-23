Validator Anchor Token Demo -- Solana PDA + Token Minting
========================================================

This project demonstrates how to build a PDA-based token minting and validator management system using Anchor and the SPL Token program.

* * * * *

📌 **Features**
---------------

-   ✅ PDA-based Profile creation

-   ✅ PDA-based global token mint and mint-authority

-   ✅ Validator account creation + token allocation

-   ✅ Secure token transfers between users

-   ✅ Token burning from user ATAs

-   ✅ Mint authority re-assignment (DAO-style logic)

-   ✅ Fully tested with Anchor + Mocha

* * * * *

🯡 **Program Architecture**
---------------------------

```
User (Signer)
   ├─▶ init_profile       → Creates profile_pda
   ├─▶ create_mint        → Sets up mint_pda and mint_auth_pda (PDA-based)
   ├─▶ init_validator     → Creates validator_pda and mints tokens to ATA
   ├─▶ transfer_tokens    → Sends tokens to another user's ATA
   ├─▶ burn_tokens        → Burns tokens from own ATA
   └─▶ reassign_mint_auth → Moves authority to new pubkey (e.g. DAO)
```

* * * * *

🗂 **Accounts Overview**
------------------------

| Account | PDA | Seeds Used | Description |
| `UserProfile` | ✅ | `["profile", user_pubkey]` | Stores user's profile name + authority |
| `ValidatorInfo` | ✅ | `["validator", user_pubkey, id_bytes]` | Validator metadata |
| `Mint` | ✅ | `["global-mint"]` | Token mint account |
| `Mint Authority` | ✅ | `["mint-authority"]` | PDA that signs mint/burn/reassign calls |
| `TokenAccount` | ❌ | Auto-generated ATA | Holds token balances per user |

* * * * *

📓 **Program Instructions**
---------------------------

### `init_profile(name: String)`

Creates a PDA-based profile.

### `create_mint()`

Initializes a new mint using PDA mint authority.

### `init_validator(id: u64, name: String)`

Creates validator PDA, allocates tokens to user ATA via PDA mint authority.

### `transfer_tokens(amount: u64)`

Transfers tokens between token accounts. Requires sender signature.

### `burn_tokens(amount: u64)`

Burns tokens from user's token account.

### `reassign_mint_authority(new_authority: Pubkey)`

Reassigns minting power to a new authority, e.g., a DAO-controlled wallet.

* * * * *

🖋️ **Testing Strategy**
------------------------

All features are tested with:

-   Mocha + Chai

-   Anchor test framework

-   Manual ATA creation + airdrop logic

-   PDA creation and validation
