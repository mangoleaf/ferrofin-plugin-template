#!/usr/bin/env bash
# Build the plugin into a ready-to-install WASM component under dist/.
#
# The only prerequisite is rustup (https://rustup.rs). The pinned toolchain
# and the wasm32-wasip2 target are installed automatically on first run,
# courtesy of rust-toolchain.toml — you do not need to add the target yourself.
set -euo pipefail
cd "$(dirname "$0")"

echo "==> Building (release, wasm32-wasip2)…"
cargo build --release --target wasm32-wasip2

# The cdylib produces exactly one .wasm; find it regardless of the crate name.
artifact="$(ls target/wasm32-wasip2/release/*.wasm | head -n1)"
mkdir -p dist
name="$(basename "$artifact")"
cp "$artifact" "dist/$name"

echo
echo "✅ Built dist/$name  ($(du -h "dist/$name" | cut -f1))"
echo
echo "Install it:"
echo "  1. Copy dist/$name into your Ferrofin data dir's plugins/ folder:"
echo "       cp dist/$name {ferrofin_data_dir}/plugins/"
echo "  2. Restart the Ferrofin server."
echo "  3. Enable the plugin in the dashboard (Dashboard → Plugins)."
