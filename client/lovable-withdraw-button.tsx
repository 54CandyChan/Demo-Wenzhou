import { useMemo, useState } from "react";
import type { Connection, PublicKey } from "@solana/web3.js";
import { exchangePoints, fetchUserPoints } from "./atx-points-exchange";

type BrowserWallet = {
  publicKey: PublicKey | null;
  signTransaction: (transaction: unknown) => Promise<unknown>;
  signAllTransactions?: (transactions: unknown[]) => Promise<unknown[]>;
};

type WithdrawButtonProps = {
  connection: Connection;
  wallet: BrowserWallet;
  programId: PublicKey;
  atxMint: PublicKey;
  className?: string;
};

export function AtxWithdrawButton({
  connection,
  wallet,
  programId,
  atxMint,
  className,
}: WithdrawButtonProps) {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [message, setMessage] = useState<string>("");
  const isConnected = useMemo(() => Boolean(wallet.publicKey), [wallet.publicKey]);

  async function handleWithdrawClick() {
    if (!wallet.publicKey) {
      setMessage("Please connect your wallet first.");
      return;
    }

    try {
      setIsSubmitting(true);
      setMessage("Checking points balance...");

      const userPoints = await fetchUserPoints({
        connection,
        walletAddress: wallet.publicKey,
        programId,
      });

      if (!userPoints || userPoints.points.toNumber() < 1000) {
        setMessage("You need at least 1000 points before withdrawing.");
        return;
      }

      setMessage("Preparing on-chain exchange. Please confirm in your wallet.");

      const signature = await exchangePoints({
        connection,
        wallet,
        programId,
        atxMint,
      });

      setMessage(`Exchange succeeded. Signature: ${signature}`);
    } catch (error) {
      const detail = error instanceof Error ? error.message : "Unknown error";
      setMessage(`Exchange failed: ${detail}`);
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div>
      <button
        type="button"
        className={className}
        disabled={!isConnected || isSubmitting}
        onClick={handleWithdrawClick}
      >
        {isSubmitting ? "Processing..." : "Withdraw"}
      </button>
      {message ? <p>{message}</p> : null}
    </div>
  );
}
