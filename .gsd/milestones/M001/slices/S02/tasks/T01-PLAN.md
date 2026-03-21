# T01: 02-grid-engine-data-model 01

**Slice:** S02 — **Milestone:** M001

## Description

Create all foundation types, configuration structs, seed module, edge types, and connector trait for the puzzle grid engine.

Purpose: Every subsequent plan depends on these types. Defining contracts first prevents the "scavenger hunt" anti-pattern where later tasks must explore to understand shapes.

Output: Config types with validation, deterministic seed hashing, edge/connector abstractions — all compiled and tested.

## Must-Haves

- [ ] "All puzzle config types exist with validation and correct defaults"
- [ ] "String seed hashing produces deterministic u64 values across runs"
- [ ] "ConnectorGenerator trait is defined and accepts EdgeParams, returns Vec<CubicBez>"
- [ ] "Unit conversion between mm and inches works correctly"

## Files

- `crates/puzzle-core/Cargo.toml`
- `crates/puzzle-core/src/config.rs`
- `crates/puzzle-core/src/seed.rs`
- `crates/puzzle-core/src/edge.rs`
- `crates/puzzle-core/src/connector.rs`
- `crates/puzzle-core/src/lib.rs`
