#!/usr/bin/env bash
# Build the playground WebAssembly module and copy it next to the page.
#
# Usage: bash playground/build.sh
set -euo pipefail

cd "$(dirname "$0")/.."

# This workspace can live on a UNC mount where cargo's incremental lock file
# cannot be created; incremental compilation is not needed for a release build.
export CARGO_INCREMENTAL=0

if ! rustup target list --installed | grep -qx "wasm32-unknown-unknown"; then
  echo "installing rust target wasm32-unknown-unknown"
  rustup target add wasm32-unknown-unknown
fi

cargo build -p rupa-wasm --target wasm32-unknown-unknown --release

cp target/wasm32-unknown-unknown/release/rupa_wasm.wasm playground/rupa_wasm.wasm

bytes=$(wc -c < playground/rupa_wasm.wasm | tr -d ' ')
echo "built playground/rupa_wasm.wasm (${bytes} bytes)"
echo "serve it with: node playground/serve.mjs"
