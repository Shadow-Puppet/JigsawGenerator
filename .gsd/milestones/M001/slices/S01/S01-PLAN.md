# S01: Build Pipeline Wasm Foundation

**Goal:** Set up the complete Rust-to-WASM-to-browser build pipeline with a working round-trip proof-of-life demo.
**Demo:** Set up the complete Rust-to-WASM-to-browser build pipeline with a working round-trip proof-of-life demo.

## Must-Haves


## Tasks

- [x] **T01: 01-build-pipeline-wasm-foundation 01** `est:6min`
  - Set up the complete Rust-to-WASM-to-browser build pipeline with a working round-trip proof-of-life demo.

Purpose: Establish the foundational infrastructure that all subsequent phases build on. Proves TypeScript can call Rust/WASM functions and display results in the browser, including error handling across the boundary.

Output: Cargo workspace (puzzle-core + puzzle-wasm), Vite web app, npm build scripts, and a browser demo where users input grid dimensions and see piece count breakdown computed by Rust/WASM.

## Files Likely Touched

- `Cargo.toml`
- `.gitignore`
- `crates/puzzle-core/Cargo.toml`
- `crates/puzzle-core/src/lib.rs`
- `crates/puzzle-wasm/Cargo.toml`
- `crates/puzzle-wasm/src/lib.rs`
- `web/package.json`
- `web/tsconfig.json`
- `web/vite.config.ts`
- `web/index.html`
- `web/src/main.ts`
- `web/src/style.css`
