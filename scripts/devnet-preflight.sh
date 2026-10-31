#!/usr/bin/env bash
set -euo pipefail

export ANCHOR_WALLET="${ANCHOR_WALLET:-$HOME/.config/solana/id.json}"
solana config set --url devnet
solana address
solana balance
anchor --version
solana --version
anchor build
anchor test --skip-local-validator
