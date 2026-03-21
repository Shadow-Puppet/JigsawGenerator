# S02: Grid Engine Data Model

**Goal:** Create all foundation types, configuration structs, seed module, edge types, and connector trait for the puzzle grid engine.
**Demo:** Create all foundation types, configuration structs, seed module, edge types, and connector trait for the puzzle grid engine.

## Must-Haves


## Tasks

- [x] **T01: 02-grid-engine-data-model 01** `est:4min`
  - Create all foundation types, configuration structs, seed module, edge types, and connector trait for the puzzle grid engine.

Purpose: Every subsequent plan depends on these types. Defining contracts first prevents the "scavenger hunt" anti-pattern where later tasks must explore to understand shapes.

Output: Config types with validation, deterministic seed hashing, edge/connector abstractions — all compiled and tested.
- [x] **T02: 02-grid-engine-data-model 02** `est:3min`
  - Implement the core grid engine with shared-edge data model using TDD — the heart of the puzzle generator.

Purpose: The grid engine is the central data structure that all future phases build upon. Shared-edge correctness is critical — if an internal edge appears twice or piece indexing is wrong, all downstream geometry will be broken. TDD ensures correctness before complexity.

Output: PuzzleGrid that constructs NxM grids with shared edges, deterministic seeded tab assignment, and piece index views.
- [x] **T03: 02-grid-engine-data-model 03** `est:2min`
  - Wire the grid engine through the WASM boundary so the browser can generate puzzle grids via JSON API.

Purpose: Completes the Phase 2 vertical slice — types, engine, and boundary all connected. Proves the full pipeline works: JSON config in -> Rust grid engine -> JSON grid data out via WASM.

Output: Working WASM endpoint that accepts puzzle config, generates grid, returns serializable grid data.

## Files Likely Touched

- `crates/puzzle-core/Cargo.toml`
- `crates/puzzle-core/src/config.rs`
- `crates/puzzle-core/src/seed.rs`
- `crates/puzzle-core/src/edge.rs`
- `crates/puzzle-core/src/connector.rs`
- `crates/puzzle-core/src/lib.rs`
- `crates/puzzle-core/src/grid.rs`
- `crates/puzzle-core/src/piece.rs`
- `crates/puzzle-core/src/lib.rs`
- `crates/puzzle-wasm/src/lib.rs`
- `crates/puzzle-wasm/Cargo.toml`
