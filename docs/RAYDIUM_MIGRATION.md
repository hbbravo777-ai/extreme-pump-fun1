# Raydium CPMM graduation integration

## Architecture

The custom Anchor bonding curve is not a Raydium LaunchLab pool. Raydium's
LaunchLab SDK migration methods operate on Raydium LaunchLab state; they cannot
be called on this project's custom curve accounts.

For this project, use Raydium SDK V2 to create a CPMM pool from the custom
curve's graduated token and SOL/WSOL reserves. The flow must be an explicit,
separately reviewed migration executor, not an unrestricted admin transfer.

## Required state machine and custody design

1. Add Trading, Paused, GraduationPending, and Graduated states.
2. At graduation, freeze trading and snapshot exact SOL/token balances.
3. Exclude SOL vault rent and all already-distributed fees; confirm reserve
   accounting equals actual vault balances.
4. Transfer the migration amounts to a one-time escrow/migration authority
   bound to mint, CPMM program ID, amounts, and destination pool parameters.
5. Create the CPMM pool on devnet with Raydium SDK V2 and confirm the transaction.
6. Verify pool mint pair, actual deposited amounts, pool ID, LP destination,
   and LP lock/burn allocation before marking Graduated.
7. Make migration retry-safe if pool creation succeeds but final status update fails.

## SDK and devnet details

Use the official SDK V2 and demo repo as the API source of truth. SDK method
signatures and devnet program IDs must be pinned and verified together; do not
hardcode mainnet program IDs into a devnet script. The official SDK demo explains
that transaction builders return execute/transaction/extInfo and that devnet
requires devnet program IDs/configuration.

## Explicitly not done

No Raydium transaction has been built or sent. No pool has been created. The
custom curve has no migration instruction yet. Do not advertise graduation as
available until escrow, authorization, LP custody, and integration tests exist.
