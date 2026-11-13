#!/usr/bin/env bash
set -euo pipefail
cd /workspace/pastimer
if [ ! -x /workspace/.cargo/bin/rustup ]; then
  installer=$(mktemp /tmp/pastimer-rustup.XXXXXX)
  trap 'rm -f "$installer"' EXIT
  curl --proto '=https' --tlsv1.2 -fsS https://sh.rustup.rs -o "$installer"
  CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup sh "$installer" -y --no-modify-path --profile minimal --default-toolchain 1.93.1
fi
. scripts/rust-env.sh
rustup show active-toolchain
cargo fetch --locked
cargo build --locked
npm ci --cache /workspace/.npm-cache
