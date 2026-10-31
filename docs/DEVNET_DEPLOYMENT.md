# Devnet build and deployment

Deployment has NOT been run from this environment. It requires the user's local
Solana keypair and funded devnet wallet.

## Toolchain

Install Rust, Solana CLI, Anchor 0.31.1, and Node.js 20+ using their official
installation instructions. Verify solana, anchor, and rustc versions.

## Generate a dedicated program ID

Do not deploy with the System Program placeholder.

```bash
mkdir -p target/deploy
solana-keygen new --outfile target/deploy/extreme_pump_fun-keypair.json
solana address -k target/deploy/extreme_pump_fun-keypair.json
```

Copy the generated public key into declare_id! in
programs/extreme_pump_fun/src/lib.rs and both program entries in Anchor.toml.
Never commit the keypair.

## Build and test

```bash
anchor build
anchor test
cargo audit
```

Resolve all failures before deployment.

## Fund and deploy to devnet

```bash
solana config set --url devnet
solana airdrop 2
solana balance
anchor keys sync
anchor build
anchor deploy --provider.cluster devnet
```

Record the deployed program ID, transaction signature, IDL, and build artifacts.
Verify the program account on Solana Explorer using the devnet cluster.

## Initialize and smoke-test

Initialize with a dedicated platform treasury. Create a classic SPL test mint,
mint test inventory to the creator ATA, and call create_curve. Exercise buy and
sell with realistic min-out and deadline values; verify balances and emitted
events. Keep all testing on devnet.

No wallet was accessed, no airdrop requested, no transaction signed, and no
deployment performed by ChatGPT.
