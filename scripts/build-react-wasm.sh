#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

cargo build --release --target wasm32-unknown-unknown --features wasm
mkdir -p pkg
wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/searchlight.wasm

cat > pkg/package.json <<'JSON'
{
  "name": "@luhanxin/searchlight",
  "version": "0.1.0",
  "description": "Searchlight WebAssembly package for browser and React/Vite local search.",
  "license": "MIT",
  "type": "module",
  "sideEffects": false,
  "module": "searchlight.js",
  "types": "searchlight.d.ts",
  "exports": {
    ".": {
      "types": "./searchlight.d.ts",
      "import": "./searchlight.js"
    }
  },
  "files": [
    "searchlight.js",
    "searchlight_bg.wasm",
    "searchlight.d.ts"
  ],
  "publishConfig": {
    "access": "public"
  }
}
JSON

printf 'Generated React/Vite WASM package at %s/pkg\n' "$ROOT_DIR"
