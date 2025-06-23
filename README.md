# smartolana

🧱 **Solana Token Workflow (PDA Based) – Anchor + SPL Token**
-------------------------------------------------------------

### 🔄 Overview

Plain textANTLR4BashCC#CSSCoffeeScriptCMakeDartDjangoDockerEJSErlangGitGoGraphQLGroovyHTMLJavaJavaScriptJSONJSXKotlinLaTeXLessLuaMakefileMarkdownMATLABMarkupObjective-CPerlPHPPowerShell.propertiesProtocol BuffersPythonRRubySass (Sass)Sass (Scss)SchemeSQLShellSwiftSVGTSXTypeScriptWebAssemblyYAMLXML`   textCopyEditUser (Signer)     |     ├──▶ [Create Profile PDA] ----------------------------▶ profile_pda     |     ├──▶ [Create Mint] (PDA-based Mint & Authority)     |       |     |       ├──▶ mint_pda (global-mint)     |       └──▶ mint_auth_pda (mint-authority)     |     ├──▶ [Create Validator PDA & ATA] --------------------▶ validator_pda     |       |     |       └──▶ Mints tokens to validator_ata via CPI     |     ├──▶ [Transfer Tokens] to other user ----------------▶ recipient_ata     |     ├──▶ [Burn Tokens] from own ATA     |     └──▶ [Reassign Mint Authority] → new pubkey   `

### 🗂 **Account & PDA Breakdown**

AccountPDA?Seeds UsedDescriptionUserProfile✅\["profile", user\_pubkey\]User’s on-chain profileValidatorInfo✅\["validator", user\_pubkey, id\_bytes\]Per-validator metadataMint✅\["global-mint"\]The SPL token mintMint Authority✅\["mint-authority"\]PDA signer that can mint tokensAssociated Token❌Deterministic via getAssociatedTokenAddressHolds user's token balance

### 🧾 **Signer Rules Recap**

ActionSignerWhy?initProfileuserCreates profile for signercreateMintpayerPays for mint accountinitValidatoruserCreator of validator, also ATAmintTo (CPI)mint\_authority (PDA)Signed via seedstransferTokensuser (as sender)Owner of the from ATAburnTokensuserBurns from their own ATAsetAuthority (CPI)mint\_authority (PDA)Signed with seeds again

### 🔁 **Authority Types**

*   MintTokens: Who can mint new tokens.
    
*   FreezeAccount: Who can freeze any ATA.
    
*   AccountOwner: Ownership of a specific ATA.
    

Used set\_authority with AuthorityType::MintTokens.