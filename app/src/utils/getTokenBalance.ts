import { getAssociatedTokenAddress, getAccount } from "@solana/spl-token";
import { PublicKey, Connection } from "@solana/web3.js";

export const getTokenBalance = async (
  connection: Connection,
  mint: PublicKey,
  owner: PublicKey
) => {
  const ata = await getAssociatedTokenAddress(mint, owner);
  const accountInfo = await getAccount(connection, ata);
  return Number(accountInfo.amount);
}; 