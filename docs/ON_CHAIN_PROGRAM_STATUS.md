# On-chain program status

The Anchor crate currently provides the first state/configuration layer only:

- A singleton platform config PDA with a fixed 100 bps (1%) platform fee.
- A curve metadata PDA keyed by the supplied mint public key.
- A creator-configurable fee in basis points, with a guard that combined fees remain below 10,000 bps.
- Creator-only creator-fee updates and emitted events.

## Important limitations

This is not a functioning token launch or trading program yet. In particular:

- `create_curve` records a mint public key but does not verify or create an SPL Token mint.
- There are no buy/sell instructions, token or SOL transfers, escrow/vault PDAs, fee withdrawals, slippage checks, graduation logic, or Raydium CPI.
- The program ID in `Anchor.toml` and `lib.rs` is a development placeholder (System Program ID). Generate a dedicated program keypair and replace it before any build intended for deployment.
- No audit, fuzzing, integration test, or devnet deployment has been completed.

## Required next steps

1. Generate and set a dedicated program ID; verify the Anchor workspace builds.
2. Add SPL Token / Token-2022 mint validation and PDA-controlled token/SOL vaults.
3. Implement atomic buy/sell with checked arithmetic, slippage bounds, rent handling, and fee transfers.
4. Add authority-controlled platform fee withdrawal and creator-only creator fee withdrawal.
5. Add graduation thresholds and a separately reviewed Raydium migration flow.
6. Add Anchor integration tests for authorization, rounding, overflow, account substitution, fee boundaries, and failed transfers.
