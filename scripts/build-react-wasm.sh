#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

cargo build --release --target wasm32-unknown-unknown --features wasm
mkdir -p pkg
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/searchlight.wasm

cat > pkg/package.json <<'JSON'
{
  "name": "searchlight",
  "version": "0.1.0",
  "type": "module",
  "module": "searchlight.js",
  "types": "searchlight.d.ts",
  "files": [
    "searchlight.js",
    "searchlight_bg.wasm",
    "searchlight.d.ts"
  ]
}
JSON

printf 'Generated React/Vite WASM package at %s/pkg\n' "$ROOT_DIR"
