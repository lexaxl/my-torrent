#!/bin/sh
set -eu

# Xcode Run Script build phases don't source shell profiles, so cargo
# (typically installed under ~/.cargo/bin via rustup) isn't on PATH by default.
export PATH="$HOME/.cargo/bin:$PATH"

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ENGINE_DIR="$ROOT_DIR/engine"
OUT_DIR="$ROOT_DIR/MyTorrent/Generated"

# Apple Silicon only (NFR2) — pinned explicitly rather than relying on the
# host machine's default cargo target.
RUST_TARGET="aarch64-apple-darwin"
TARGET_DIR="$ENGINE_DIR/target/$RUST_TARGET/release"

cd "$ENGINE_DIR"

cargo build --release --target "$RUST_TARGET"

mkdir -p "$OUT_DIR"
# `--target` here matters even though it's the host's own triple: Cargo keys
# its output directory on whether `--target` was passed at all, not on triple
# equality, so omitting it (as this used to) made `uniffi-bindgen` itself
# build against a separate `target/release/` tree — recompiling the entire
# engine dependency graph (librqbit, tokio, etc.) a second time from scratch
# on every build. Passing it here reuses the same `$TARGET_DIR` the `cargo
# build` above already populated (code-review fix, Story 4.3).
cargo run --quiet --release --target "$RUST_TARGET" --bin uniffi-bindgen -- \
  generate \
  --library "$TARGET_DIR/libengine.dylib" \
  --language swift \
  --out-dir "$OUT_DIR"

mv -f "$OUT_DIR/engineFFI.modulemap" "$OUT_DIR/module.modulemap"
