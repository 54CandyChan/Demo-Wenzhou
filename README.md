# AVAX Points Exchange

This project contains an Anchor 0.29.0 Solana contract scaffold with the corrected exchange rule:

- `1000 points = 0.001 AVAX`

Included files:

- `programs/avax_points_exchange/src/lib.rs`: Solana Rust smart contract
- `client/avax-points-exchange.ts`: frontend helper for wallet calls
- `client/lovable-avax-withdraw-button.tsx`: Lovable/React withdraw button example

## Implemented Features

- `initialize_config`
- `add_user_points`
- `sub_user_points`
- `toggle_pause`
- `exchange_points`
- `withdraw_tokens`

## Important Notes

1. Solana programs do not directly return dynamic arrays like a normal backend API.
   Use account reads from `client/avax-points-exchange.ts` for:
   - `get_user_points(user_pubkey)`
   - `get_exchange_records(user_pubkey)`
   - `get_global_config()`

2. The current Program ID is still a placeholder and must be replaced before deployment.
   Update both:
   - `programs/avax_points_exchange/src/lib.rs`
   - `Anchor.toml`

3. `initialize_config` creates the program vault token account for the AVAX SPL token mint.
   The admin must deposit enough AVAX tokens into that vault before users can exchange points.

## Build And Deploy

### 1. Install Dependencies

- Rust 1.75+
- Solana CLI
- Anchor CLI 0.29.0

### 2. Build

```bash
anchor build
```

### 3. Deploy

```bash
anchor deploy
```

After deployment, replace the generated Program ID in:

- `programs/avax_points_exchange/src/lib.rs`
- `Anchor.toml`
- your frontend environment config

### 4. Initialize Config

After deployment, call `initialize_config` with:

- `avax_mint`

### 5. Lovable Button Integration

Import:

```tsx
import { AvaxWithdrawButton } from "./client/lovable-avax-withdraw-button";
```

Pass these props:

- `connection`
- `wallet`
- `programId`
- `avaxMint`

When the button is clicked it will:

1. Check the current wallet points balance
2. Verify the wallet has at least `1000` points
3. Request wallet signature
4. Call on-chain `exchange_points`

## Suggested Next Steps

- Add Anchor tests for insufficient points, paused state, and low vault balance
- Wrap admin instructions behind a backend service instead of exposing them in the browser
- Replace the placeholder Program ID, mint address, and RPC settings before production
