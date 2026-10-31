# Extreme Pump.Fun

An original Solana token-launchpad project. This repository contains an early Anchor
program draft, a responsive React/Vite wallet-connected web shell, fee/curve simulation
helpers, a guarded Raydium SDK V2 devnet CPMM example, and deployment/security docs.

**This is not production-ready and must not be used to accept public funds.**

## Web app

Requirements: Node.js 20+ and npm.

    npm install
    cp .env.example .env
    npm run web:dev
    npm run web:build

The web shell supports Phantom and Solflare wallet connection on devnet. Token
discovery, token creation, and trading are intentionally not enabled until a verified
program deployment, generated IDL/client, indexer, and end-to-end tests exist.

For static hosting, the included vercel.json builds dist-web. Configure
VITE_SOLANA_CLUSTER, VITE_RPC_URL, and (only after verification)
VITE_PROGRAM_ID in the hosting provider's environment settings.

## Anchor program

The Anchor program draft contains:
- PDA-derived platform config, curve state, SOL vault, and curve-owned SPL token vault.
- 1% platform fee and a separate per-curve creator fee.
- Buy/sell constant-product quote calculations with checked integer arithmetic.
- Minimum output and transaction deadline checks.
- Creator fee update authorization and admin pause.

The program ID is deliberately a placeholder. Generate a dedicated keypair, update
declare_id! and Anchor.toml, then build and test using docs/DEVNET_DEPLOYMENT.md.
Do not commit keypairs.

## Raydium

The custom curve is not Raydium LaunchLab. The Raydium SDK V2 CPMM example creates
a devnet pool from balances already held by the configured signer. It does not yet
migrate this program's PDA-held reserves or implement graduation. See
docs/RAYDIUM_MIGRATION.md.

## Security and release

Read docs/SECURITY_REVIEW.md and docs/PRODUCTION_READINESS.md. These are engineering
checklists, not an independent audit. No successful build, validator test, deployment,
or independent audit is claimed. Mainnet use is blocked pending the listed release gates.
