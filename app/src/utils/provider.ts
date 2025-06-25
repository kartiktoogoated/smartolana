import { AnchorProvider, Program, setProvider } from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import idl from "../../../target/idl/smartolana.json";
import type { ValidatorAnchorDemo } from "../../../target/types/smartolana";

const programID: PublicKey = new PublicKey("BH2vhWg3AJqKn5VXKf6nepTPQUigJEhPEApUo9XXekjz");

export const getAnchorProvider = (wallet: any) => {
  if (!wallet || !wallet.publicKey || typeof wallet.signTransaction !== "function") {
    throw new Error("Wallet not connected or invalid. Please connect your wallet.");
  }
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const provider: AnchorProvider = new AnchorProvider(connection, wallet, { commitment: "confirmed" });
  setProvider(provider);

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const program = new Program<ValidatorAnchorDemo>(idl as any, programID as any, provider as any);

  return { provider, program };
};
