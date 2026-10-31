# Production readiness gate

This document is a release checklist, not a claim that the product is production-ready.

## Current implementation snapshot

- [x] Repository has a TypeScript fee/curve simulation foundation.
- [x] Anchor program draft includes create-curve inventory deposit, buy, sell,
  immediate platform/creator fee transfers, min-output checks, deadlines,
  PDA-derived vaults, and admin pause.
- [x] Responsive Vite/React landing and wallet-connected web shell (Phantom/Solflare).
- [x] Raydium SDK V2 devnet CPMM pool creation example with explicit signing opt-in.
- [x] Deployment, security, and Raydium architecture documentation.
- [x] GitHub Actions workflows authored for Anchor and frontend build checks.

## Not implemented / release blockers

- [ ] Verified non-placeholder program ID and successful reproducible Anchor build.
- [ ] Anchor integration tests for all instructions and adversarial account cases.
- [ ] Independent Solana security audit and remediation sign-off.
- [ ] Token-mint creation flow and metadata storage/validation.
- [ ] Generated Anchor IDL/client wired to the frontend.
- [ ] Real create-token, buy, sell, fee dashboards, transaction simulation, and
  confirmation UX. Wallet connection is present; trading UI is not connected.
- [ ] API/indexer/database for token discovery, trades, holder snapshots, and charts.
- [ ] Rate limiting, input validation, abuse prevention, monitoring, alerting,
  backups, and incident response.
- [ ] Raydium graduation state machine, escrow, one-time migration authorization,
  LP custody/lock policy, retry reconciliation, and integration tests.
- [ ] Devnet end-to-end deployment and smoke tests using an owner-controlled wallet.
- [ ] Mainnet configuration, multisig-controlled upgrade authority, legal/compliance
  review, privacy policy, terms, risk disclosures, and support operations.

## Deployment boundary

The web shell can be deployed as a static Vite site after its build succeeds.
Do not enable public token creation or real-money trading from this shell.
The program source currently contains a placeholder ID and is not verified as
compilable or audited. Devnet deployment must be performed only after the program
ID is generated, CI build succeeds, and integration tests pass.

Mainnet launch is blocked until every relevant item above is closed and independently
reviewed. Never commit wallet keypairs, seed phrases, or .env files.
