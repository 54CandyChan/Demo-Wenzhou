import {
  AnchorProvider,
  BN,
  Program,
  type Idl,
  web3,
} from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import type { Connection, PublicKey, TransactionSignature } from "@solana/web3.js";
import { SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { AVAX_POINTS_EXCHANGE_IDL } from "./avax-points-exchange.idl";

type BrowserWallet = {
  publicKey: PublicKey | null;
  signTransaction: (transaction: web3.Transaction) => Promise<web3.Transaction>;
  signAllTransactions?: (transactions: web3.Transaction[]) => Promise<web3.Transaction[]>;
};

const encoder = new TextEncoder();
const GLOBAL_CONFIG_SEED = encoder.encode("global-config");
const USER_POINTS_SEED = encoder.encode("user-points");
const EXCHANGE_RECORD_SEED = encoder.encode("exchange-record");
const VAULT_AUTHORITY_SEED = encoder.encode("vault-authority");

export function getExchangeProgram(
  connection: Connection,
  wallet: BrowserWallet,
  programId: PublicKey,
  idl: Idl = AVAX_POINTS_EXCHANGE_IDL,
) {
  const provider = new AnchorProvider(connection, wallet as never, {
    commitment: "confirmed",
  });

  return new Program(idl, programId, provider);
}

export async function deriveExchangeAddresses(
  programId: PublicKey,
  user: PublicKey,
  avaxMint: PublicKey,
  connection?: Connection,
  idl: Idl = AVAX_POINTS_EXCHANGE_IDL,
) {
  const [configPda] = PublicKey.findProgramAddressSync([GLOBAL_CONFIG_SEED], programId);
  const [userPointsPda] = PublicKey.findProgramAddressSync(
    [USER_POINTS_SEED, user.toBuffer()],
    programId,
  );

  let exchangeCounter = new BN(0);
  if (connection) {
    const provider = new AnchorProvider(
      connection,
      {
        publicKey: user,
        signTransaction: async (tx) => tx,
      } as never,
      { commitment: "confirmed" },
    );
    const program = new Program(idl, programId, provider);
    const config = await program.account.globalConfig.fetchNullable(configPda);
    if (config) {
      exchangeCounter = config.exchangeCounter as BN;
    }
  }

  const [exchangeRecordPda] = PublicKey.findProgramAddressSync(
    [EXCHANGE_RECORD_SEED, Uint8Array.from(exchangeCounter.toArray("le", 8))],
    programId,
  );
  const [vaultAuthorityPda] = PublicKey.findProgramAddressSync(
    [VAULT_AUTHORITY_SEED, configPda.toBuffer()],
    programId,
  );

  const vaultTokenAccount = getAssociatedTokenAddressSync(avaxMint, vaultAuthorityPda, true);
  const userTokenAccount = getAssociatedTokenAddressSync(avaxMint, user);

  return {
    configPda,
    userPointsPda,
    exchangeRecordPda,
    vaultAuthorityPda,
    vaultTokenAccount,
    userTokenAccount,
  };
}

export async function exchangePoints(params: {
  connection: Connection;
  wallet: BrowserWallet;
  programId: PublicKey;
  avaxMint: PublicKey;
  idl?: Idl;
}): Promise<TransactionSignature> {
  const { connection, wallet, programId, avaxMint, idl = AVAX_POINTS_EXCHANGE_IDL } = params;

  if (!wallet.publicKey) {
    throw new Error("Wallet not connected.");
  }

  const program = getExchangeProgram(connection, wallet, programId, idl);
  const addresses = await deriveExchangeAddresses(
    programId,
    wallet.publicKey,
    avaxMint,
    connection,
    idl,
  );

  return program.methods
    .exchangePoints()
    .accounts({
      user: wallet.publicKey,
      config: addresses.configPda,
      userPoints: addresses.userPointsPda,
      exchangeRecord: addresses.exchangeRecordPda,
      avaxMint,
      vaultAuthority: addresses.vaultAuthorityPda,
      vaultTokenAccount: addresses.vaultTokenAccount,
      userTokenAccount: addresses.userTokenAccount,
      systemProgram: web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
    })
    .rpc();
}

export async function fetchUserPoints(params: {
  connection: Connection;
  walletAddress: PublicKey;
  programId: PublicKey;
  idl?: Idl;
}) {
  const { connection, walletAddress, programId, idl = AVAX_POINTS_EXCHANGE_IDL } = params;
  const [userPointsPda] = PublicKey.findProgramAddressSync(
    [USER_POINTS_SEED, walletAddress.toBuffer()],
    programId,
  );
  const provider = new AnchorProvider(
    connection,
    {
      publicKey: walletAddress,
      signTransaction: async (tx) => tx,
    } as never,
    { commitment: "confirmed" },
  );
  const program = new Program(idl, programId, provider);
  return program.account.userPoints.fetchNullable(userPointsPda);
}

export async function fetchExchangeRecords(params: {
  connection: Connection;
  walletAddress: PublicKey;
  programId: PublicKey;
  idl?: Idl;
}) {
  const { connection, walletAddress, programId, idl = AVAX_POINTS_EXCHANGE_IDL } = params;
  const provider = new AnchorProvider(
    connection,
    {
      publicKey: walletAddress,
      signTransaction: async (tx) => tx,
    } as never,
    { commitment: "confirmed" },
  );
  const program = new Program(idl, programId, provider);
  return program.account.exchangeRecord.all([
    {
      memcmp: {
        offset: 8,
        bytes: walletAddress.toBase58(),
      },
    },
  ]);
}
