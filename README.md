# Extreme Pump.Fun

An original Solana token-launch platform project with planned Raydium liquidity migration.
This repository is an early engineering scaffold, not a production-ready launchpad.

## Foundation added

- Strict TypeScript project setup.
- Explicit basis-point fee accounting.
- Separate 1% platform fee and configurable creator fee.
- Integer-only constant-product quote helper for local simulation.
- Raydium SDK V2 dependency for the future migration adapter.

## Fee model

Both fees are calculated independently against gross input:

- Platform fee: 100 bps (1%).
- Creator fee: configurable using CREATOR_FEE_BPS. The example value is 100 bps (1%),
  provided as a starting point and not a finalized business or legal decision.
- Net curve input: gross input minus both fees.

Fee collection, authority checks, rounding, refunds, and withdrawals must be enforced
by the on-chain program, not trusted to the frontend.

## Development

Requirements: Node.js 20+ and npm.

    npm install
    npm run typecheck
    npm run build

Copy .env.example to .env. Never commit private keys, seed phrases, RPC credentials,
or production authority secrets.

## Planned milestones

1. Audited Anchor program: token creation, bonding curve state, buy/sell, fee vaults,
   creator/platform fee withdrawals, pause controls, and account constraints.
2. API/indexer: transaction status, token metadata, trades, holders, and live curve data.
3. Web app: wallet adapter, launch form, token discovery, chart/trade panel, creator dashboard.
4. Raydium migration adapter: detect graduation, calculate liquidity, create the approved
   pool type with Raydium SDK V2, and verify resulting pool state.
5. Devnet integration tests, security review, and staged mainnet rollout.

## Safety status

No mainnet deployment, wallet custody, live trading, or automatic liquidity migration
is enabled. The quote helper is simulation-only. Production behavior requires an audited
on-chain implementation and devnet integration tests.
