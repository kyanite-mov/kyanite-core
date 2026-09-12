#!/bin/bash

# Verify all Rust code, tests, and docs build correctly.

set -Eeuo pipefail
trap 'printf "ERROR: command failed with exit status %d at line %d\n" "$?" "$LINENO" >&2' ERR

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

printf 'Automatically fixing and verifying Rust code.\n'
read -rp 'Press Enter to continue, Ctrl+C to exit... ' || exit 1

export RUSTDOCFLAGS="${RUSTDOCFLAGS:-} -D warnings"

cargo fmt --all
cargo clippy --fix --allow-dirty --allow-staged --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --release --all-features
cargo doc --locked --workspace --no-deps --all-features

printf 'All checks passed.\n'
