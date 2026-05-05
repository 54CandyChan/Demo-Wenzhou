import type { Idl } from "@coral-xyz/anchor";

export const ATX_POINTS_EXCHANGE_IDL: Idl = {
  version: "0.1.0",
  name: "atx_points_exchange",
  instructions: [
    {
      name: "exchangePoints",
      accounts: [
        { name: "user", isMut: true, isSigner: true },
        { name: "config", isMut: true, isSigner: false },
        { name: "userPoints", isMut: true, isSigner: false },
        { name: "exchangeRecord", isMut: true, isSigner: false },
        { name: "atxMint", isMut: false, isSigner: false },
        { name: "vaultAuthority", isMut: false, isSigner: false },
        { name: "vaultTokenAccount", isMut: true, isSigner: false },
        { name: "userTokenAccount", isMut: true, isSigner: false },
        { name: "systemProgram", isMut: false, isSigner: false },
        { name: "tokenProgram", isMut: false, isSigner: false },
        { name: "associatedTokenProgram", isMut: false, isSigner: false },
        { name: "rent", isMut: false, isSigner: false }
      ],
      args: []
    }
  ],
  accounts: [
    {
      name: "globalConfig",
      type: {
        kind: "struct",
        fields: [
          { name: "owner", type: "publicKey" },
          { name: "isPaused", type: "bool" },
          { name: "pointsPerExchange", type: "u64" },
          { name: "atxPerExchange", type: "u64" },
          { name: "atxMint", type: "publicKey" },
          { name: "exchangeCounter", type: "u64" }
        ]
      }
    },
    {
      name: "userPoints",
      type: {
        kind: "struct",
        fields: [
          { name: "owner", type: "publicKey" },
          { name: "points", type: "u64" },
          { name: "exchangeCount", type: "u64" },
          { name: "lastExchangeTime", type: "i64" }
        ]
      }
    },
    {
      name: "exchangeRecord",
      type: {
        kind: "struct",
        fields: [
          { name: "user", type: "publicKey" },
          { name: "pointsUsed", type: "u64" },
          { name: "atxReceived", type: "u64" },
          { name: "timestamp", type: "i64" },
          { name: "exchangeId", type: "u64" }
        ]
      }
    }
  ],
  events: [
    {
      name: "PointsExchanged",
      fields: [
        { name: "exchangeId", type: "u64", index: false },
        { name: "user", type: "publicKey", index: false },
        { name: "pointsUsed", type: "u64", index: false },
        { name: "atxReceived", type: "u64", index: false },
        { name: "timestamp", type: "i64", index: false }
      ]
    }
  ],
  errors: [
    { code: 6000, name: "InsufficientPoints", msg: "Insufficient points for this exchange." },
    { code: 6001, name: "ExchangePaused", msg: "The exchange feature is currently paused." },
    { code: 6002, name: "NotOwner", msg: "Only the contract owner can perform this action." },
    { code: 6003, name: "TransferFailed", msg: "ATX token transfer failed." },
    { code: 6004, name: "InvalidAddress", msg: "Invalid wallet or mint address." },
    { code: 6005, name: "ConfigNotInitialized", msg: "The contract configuration has not been initialized." },
    { code: 6006, name: "MathOverflow", msg: "Math overflow or underflow detected." },
    { code: 6007, name: "UnauthorizedUser", msg: "The caller is not authorized to operate on this user account." },
    { code: 6008, name: "InvalidMint", msg: "The provided mint does not match the configured ATX mint." },
    { code: 6009, name: "InvalidPointsAmount", msg: "Points amount must be greater than zero." },
    { code: 6010, name: "InvalidTokenAmount", msg: "Token amount must be greater than zero." }
  ]
} as Idl;
