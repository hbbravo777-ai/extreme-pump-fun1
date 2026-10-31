# Security review — pre-audit engineering checklist

Status: NOT an independent audit. No external auditor has reviewed this code.
Do not deploy mainnet or accept public funds until the checklist is complete and
the exact deployable program has been independently audited.

## Controls intended in this iteration

- PDA-seeded config, curve, and SOL vault accounts.
- Curve mint binding and associated-token-account constraints.
- Creator-only creator-fee updates; admin-only pause.
- Platform treasury bound to initialized config.
- Separate 100 bps platform fee and per-curve creator fee.
- Checked arithmetic, reserve bounds, min-output slippage checks, deadlines.
- SOL vault rent-floor check on sells.
- Events for initialization, curve creation, trades, fee changes, and pause.

## Blocking findings and unresolved risks

1. Program ID remains the System Program placeholder. Generate a unique program
   keypair and update Anchor.toml and declare_id! before any build/deploy.
2. Code has not been compiled or executed here. Treat it as unverified until CI
   and local-validator integration tests pass.
3. pause_curve currently marks a curve complete permanently; add distinct
   Trading/Paused/GraduationPending/Graduated states and reviewed transitions.
4. No graduation instruction or Raydium CPMM migration is implemented in Anchor.
5. Fee rounding floors each fee independently. Add boundary and conservation tests.
6. Only classic SPL Token is supported. Token-2022 extensions, transfer fees,
   hooks, freeze authority, and metadata policy are not handled.
7. No recovery mechanism for a compromised creator, invalid mint, or upgrade key.
   Define multisig custody and upgrade-authority policy.
8. Account aliasing, CPI rollback, rent edge cases, direct lamport mutation, and
   token-account closure must be reviewed and tested.
9. No fuzz/property tests, compute profiling, cargo audit, or dependency review
   has been completed.

## Required pre-mainnet gates

- Replace placeholder program ID and verify reproducible build.
- Add tests for curve creation, buy/sell, wrong mint/vault/treasury, unauthorized
  actions, deadlines, slippage, depleted reserves, arithmetic boundaries, pause,
  and failed-CPI rollback.
- Add invariant/fuzz tests for reserve solvency and fee conservation.
- Run anchor build, anchor test, cargo audit, cargo deny, and local validator tests.
- Commission an independent Solana program audit against the exact deployed commit.
- Publish program address, verified source, upgrade authority policy, fee policy,
  and emergency response plan.
