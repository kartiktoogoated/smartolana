/* eslint-disable @typescript-eslint/no-explicit-any */
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useState, type FormEvent } from "react";
import { useAnchorWallet } from "@solana/wallet-adapter-react";
import { mintToValidator } from "@/utils/mintTokens";
import { createMintFromAnchor } from "@/utils/createMintFromAnchor";
import type { Wallet } from "@coral-xyz/anchor/dist/cjs/provider";

function App() {
  const [amount, setAmount] = useState("");
  const [mintAddress, setMintAddress] = useState<string | null>(null);
  const wallet = useAnchorWallet();

  const handleMint = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!wallet) {
      alert("Connect your wallet first!");
      return;
    }

    const anchorWallet = wallet as any;

    try {
      const mint = await createMintFromAnchor(anchorWallet);
      setMintAddress(mint); // store in state
      await mintToValidator(anchorWallet, mint, parseInt(amount));
      alert("✅ Tokens minted and validator initialized.");
    } catch (err) {
      console.error("❌ Minting failed:", err);
      alert("❌ Error occurred. Check console for details.");
    }
  };

  return (
    <div className="min-h-screen w-full flex flex-col items-center justify-center bg-black text-white px-2 font-sans">
      <div className="mb-8 flex flex-col items-center">
        <span className="text-3xl font-extrabold tracking-tight text-white text-center">Validator Token Minting</span>
        <span className="text-base text-neutral-400 mt-2 text-center max-w-md">
          Connect your wallet to mint and manage your validator tokens on Solana Devnet.
        </span>
      </div>

      <Card className="w-full max-w-md bg-black border border-neutral-800 shadow-lg rounded-2xl p-8 text-white">
        <CardHeader className="flex flex-col items-center mb-2">
          <WalletMultiButton className="mb-4 w-full !bg-white !text-black !rounded-lg !font-semibold !py-2 !px-4 !shadow transition" />
        </CardHeader>

        <CardContent className="flex flex-col gap-4">
          <form className="w-full flex flex-col gap-4" onSubmit={handleMint}>
            <div className="flex flex-col gap-2">
              <Label htmlFor="amount" className="text-sm font-medium text-white">Amount</Label>
              <Input
                id="amount"
                type="number"
                min="1"
                placeholder="Enter amount"
                value={amount}
                onChange={e => setAmount(e.target.value)}
                className="bg-black border border-neutral-800 text-white placeholder:text-neutral-500 rounded-md px-3 py-2"
                required
              />
            </div>

            <Button type="submit" className="mt-2 w-full bg-white text-black font-bold text-base rounded-md shadow hover:bg-neutral-200 transition py-2">
              Mint Tokens
            </Button>
          </form>

          {mintAddress && (
            <div className="text-sm text-green-400 break-all mt-4">
              ✅ Mint Address: {mintAddress}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

export default App;
