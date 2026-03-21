# T03: 02-grid-engine-data-model 03

**Slice:** S02 — **Milestone:** M001

## Description

Wire the grid engine through the WASM boundary so the browser can generate puzzle grids via JSON API.

Purpose: Completes the Phase 2 vertical slice — types, engine, and boundary all connected. Proves the full pipeline works: JSON config in -> Rust grid engine -> JSON grid data out via WASM.

Output: Working WASM endpoint that accepts puzzle config, generates grid, returns serializable grid data.

## Must-Haves

- [ ] "WASM boundary accepts full puzzle config JSON and returns grid data"
- [ ] "Seed string passed through WASM produces deterministic grid output"
- [ ] "Existing compute_pieces endpoint still works (backward compatible)"
- [ ] "WASM module builds successfully with wasm-pack"

## Files

- `crates/puzzle-wasm/src/lib.rs`
- `crates/puzzle-wasm/Cargo.toml`
