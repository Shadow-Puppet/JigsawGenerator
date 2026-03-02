---
phase: 02-grid-engine-data-model
plan: 02
subsystem: engine
tags: [grid, shared-edge, puzzle-grid, piece, deterministic, chacha8, kurbo]

# Dependency graph
requires:
  - phase: 02-grid-engine-data-model
    provides: PuzzleConfig, Edge, TabDirection, create_rng, hash_seed
provides:
  - PuzzleGrid with shared-edge h_edges/v_edges construction
  - Piece, PieceType, PieceEdges types for piece indexing
  - Deterministic grid layout from seeded RNG
  - piece_edges() for shared-edge index lookups
  - pieces() for full piece enumeration with type classification
affects: [03-connector-generation-svg-export]

# Tech tracking
tech-stack:
  added: []
  patterns: [shared-edge data model (edges stored once referenced by index), assign_direction helper for border/internal edge logic, row-major RNG consumption order for determinism]

key-files:
  created:
    - crates/puzzle-core/src/grid.rs
    - crates/puzzle-core/src/piece.rs
  modified:
    - crates/puzzle-core/src/lib.rs

key-decisions:
  - "Shared-edge model with index references: pieces reference edges by index into h_edges/v_edges arrays, not by owning Edge values"
  - "Fixed RNG consumption order: all h_edges row-major then all v_edges row-major, ensuring seed determinism"
  - "Border edges always direction=In (unused but consistent), internal edges random from RNG"

patterns-established:
  - "Shared-edge indexing: top=row*cols+col, bottom=(row+1)*cols+col, left=row*(cols+1)+col, right=row*(cols+1)+(col+1)"
  - "assign_direction() helper encapsulates border vs internal edge logic"

requirements-completed: [GRID-01, GRID-06]

# Metrics
duration: 3min
completed: 2026-03-02
---

# Phase 2 Plan 2: Grid Engine Summary

**PuzzleGrid with shared-edge h_edges/v_edges arrays, deterministic seeded tab assignment, piece indexing by edge index, and PieceType classification matching compute_piece_breakdown**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-02T23:51:37Z
- **Completed:** 2026-03-02T23:54:57Z
- **Tasks:** 3 (RED, GREEN, REFACTOR)
- **Files modified:** 3

## Accomplishments
- PuzzleGrid::new() constructs shared-edge arrays with correct counts for any valid NxM grid
- Adjacent pieces proven to share exact same edge index (shared-edge invariant)
- Same seed string produces identical grid state; different seeds produce different tab directions
- Piece type classification (corner/edge/interior) counts match existing compute_piece_breakdown for all tested grid sizes
- 23 new comprehensive tests covering edge counts, coordinates, borders, determinism, shared-edge proof, and piece types

## Task Commits

Each task was committed atomically:

1. **RED: Failing tests for PuzzleGrid** - `dc932cb` (test)
2. **GREEN: Implement PuzzleGrid::new()** - `caaae59` (feat)
3. **REFACTOR: Extract assign_direction helper** - `5653845` (refactor)

## Files Created/Modified
- `crates/puzzle-core/src/grid.rs` - PuzzleGrid struct with new(), h_edge(), v_edge(), piece_edges(), piece_type(), pieces()
- `crates/puzzle-core/src/piece.rs` - PieceEdges, PieceType, Piece types
- `crates/puzzle-core/src/lib.rs` - Added grid and piece module declarations and re-exports

## Decisions Made
- Shared-edge model uses index references (usize into Vec<Edge>) rather than cloning Edge values, ensuring true single-source-of-truth for each edge
- RNG consumed in fixed order (h_edges row-major, then v_edges row-major) to guarantee deterministic tab assignment from any seed
- Border edges set to TabDirection::In (consistent default, unused by connector generation)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- PuzzleGrid ready for connector generation (Phase 3) — edges have connector: None, ready for ConnectorGenerator to populate
- Piece indexing ready for SVG export — pieces() returns all pieces with edge indices for path generation
- One more plan remaining in Phase 2 (Plan 03)

## Self-Check: PASSED

- All 3 created/modified files verified on disk
- All 3 task commits (dc932cb, caaae59, 5653845) verified in git history
- 65/65 tests passing
- puzzle-wasm compiles without changes

---
*Phase: 02-grid-engine-data-model*
*Completed: 2026-03-02*
