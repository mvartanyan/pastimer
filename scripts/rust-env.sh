#!/usr/bin/env bash
# Optional activation for this managed cloud; ordinary Rust installations use their own PATH.
if [ -x /workspace/.cargo/bin/cargo ]; then
  export CARGO_HOME="${CARGO_HOME:-/workspace/.cargo}"
  export RUSTUP_HOME="${RUSTUP_HOME:-/workspace/.rustup}"
  export PATH="$CARGO_HOME/bin:$PATH"
  export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"
fi
