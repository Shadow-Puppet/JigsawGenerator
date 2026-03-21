# T02: 02-grid-engine-data-model 02

**Slice:** S02 — **Milestone:** M001

## Description

Implement the core grid engine with shared-edge data model using TDD — the heart of the puzzle generator.

Purpose: The grid engine is the central data structure that all future phases build upon. Shared-edge correctness is critical — if an internal edge appears twice or piece indexing is wrong, all downstream geometry will be broken. TDD ensures correctness before complexity.

Output: PuzzleGrid that constructs NxM grids with shared edges, deterministic seeded tab assignment, and piece index views.

## Must-Haves

- [ ] "PuzzleGrid constructs correct shared-edge arrays for any valid NxM grid"
- [ ] "Each internal edge exists exactly once in memory (shared-edge guarantee)"
- [ ] "Same seed produces identical grid layouts and tab directions across runs"
- [ ] "Piece at (row, col) references correct edges from shared arrays"
- [ ] "Piece breakdown reports accurate counts for any grid config"

## Files

- `crates/puzzle-core/src/grid.rs`
- `crates/puzzle-core/src/piece.rs`
- `crates/puzzle-core/src/lib.rs`
