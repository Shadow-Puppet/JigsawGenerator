# T01: 01-build-pipeline-wasm-foundation 01

**Slice:** S01 — **Milestone:** M001

## Description

Set up the complete Rust-to-WASM-to-browser build pipeline with a working round-trip proof-of-life demo.

Purpose: Establish the foundational infrastructure that all subsequent phases build on. Proves TypeScript can call Rust/WASM functions and display results in the browser, including error handling across the boundary.

Output: Cargo workspace (puzzle-core + puzzle-wasm), Vite web app, npm build scripts, and a browser demo where users input grid dimensions and see piece count breakdown computed by Rust/WASM.

## Must-Haves

- [ ] "Running `npm run build` from web/ compiles Rust to WASM and bundles the web app in a single command"
- [ ] "TypeScript calls a Rust/WASM function with grid dimensions and displays the returned piece count breakdown in the browser"
- [ ] "Invalid inputs (0, negatives) produce error messages displayed in the browser (error flow works across WASM boundary)"
- [ ] "WASM bundle is under 500KB gzipped in release build"
- [ ] "`cargo test` in puzzle-core passes all unit tests for piece count logic"

## Files

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
