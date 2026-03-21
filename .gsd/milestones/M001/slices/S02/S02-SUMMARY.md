---
id: S02
parent: M001
milestone: M001
provides:
  - PuzzleConfig with validation (rows, cols, width, height, unit, tab, jitter, border, seed)
  - Unit enum with mm/inches conversion
  - TabConfig, JitterConfig, BorderConfig with defaults and bounds
  - hash_seed FNV-1a for deterministic string-to-u64 hashing
  - create_rng for seeded ChaCha8Rng from string
  - TabDirection enum, Edge struct with length(), EdgeParams struct
  - ConnectorGenerator trait with generate() and validate()
  - NullConnector stub for testing
  - PuzzleGrid with shared-edge h_edges/v_edges construction
  - Piece, PieceType, PieceEdges types for piece indexing
  - Deterministic grid layout from seeded RNG
  - piece_edges() for shared-edge index lookups
  - pieces() for full piece enumeration with type classification
  - generate_grid WASM endpoint accepting PuzzleConfig JSON
  - GridResponse with piece breakdown, edge summary, and per-piece info
  - JSON-in/JSON-out WASM boundary for browser grid generation
requires: []
affects: []
key_files: []
key_decisions:
  - FNV-1a hash for string-to-u64 seed conversion (portable, deterministic, not std DefaultHasher)
  - rand with default-features=false to avoid getrandom panic on wasm32-unknown-unknown
  - serde_json as dev-dependency only (testing serialization without production overhead in puzzle-core)
  - RNG passed as &mut param to ConnectorGenerator rather than trait owning state (grid controls determinism)
  - Shared-edge model with index references: pieces reference edges by index into h_edges/v_edges arrays, not by owning Edge values
  - Fixed RNG consumption order: all h_edges row-major then all v_edges row-major, ensuring seed determinism
  - Border edges always direction=In (unused but consistent), internal edges random from RNG
  - WASM response types (GridResponse, PieceInfo) are WASM-layer concern, not puzzle-core types
  - Empty seed defaults to 'default' string since WASM has no OS entropy (getrandom); JS will generate random seeds in Phase 4
  - Response excludes full edge geometry (bezier control points) — added in Phase 3 with connectors
patterns_established:
  - Unit conversion at boundary: all internal math in mm, convert at input/output
  - Config validation: separate validate() methods per sub-config, PuzzleConfig::validate() chains all
  - ConnectorGenerator strategy pattern: trait object enables pluggable connector shapes
  - NullConnector pattern: minimal stub implementation for testing without real connector logic
  - Shared-edge indexing: top=row*cols+col, bottom=(row+1)*cols+col, left=row*(cols+1)+col, right=row*(cols+1)+(col+1)
  - assign_direction() helper encapsulates border vs internal edge logic
  - WASM endpoints: JSON in → deserialize to puzzle-core types → call engine → serialize WASM response types → JSON out
  - Error format: {\"error\": \"message\"} for all WASM endpoints
observability_surfaces: []
drill_down_paths: []
duration: 2min
verification_result: passed
completed_at: 2026-03-03
blocker_discovered: false
---
# S02: Grid Engine Data Model

**# Phase 2 Plan 1: Foundation Types Summary**

## What Happened

# Phase 2 Plan 1: Foundation Types Summary

**PuzzleConfig with validation, FNV-1a seed hashing to ChaCha8Rng, Edge/EdgeParams types, and ConnectorGenerator trait with NullConnector stub**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-02T23:44:06Z
- **Completed:** 2026-03-02T23:48:06Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- PuzzleConfig with full validation (2-100 rows/cols, 15-45% tab, 0-100% jitter, 0-10mm border radius)
- Unit enum with mm/inches conversion at input boundary (all internal math in mm)
- Deterministic FNV-1a hash_seed and create_rng for portable seeded ChaCha8Rng
- ConnectorGenerator trait with generate()/validate() methods and NullConnector for testing
- 42 total tests passing (20 config, 6 seed, 9 edge, 3 connector, 7 existing grid)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add dependencies and create config + seed modules** - `4c685d4` (feat)
2. **Task 2: Create edge types, connector trait, and wire module exports** - `5e4cad7` (feat)

## Files Created/Modified
- `crates/puzzle-core/Cargo.toml` - Added kurbo, rand, rand_chacha dependencies; serde_json dev-dep
- `crates/puzzle-core/src/config.rs` - Unit, PuzzleConfig, TabConfig, JitterConfig, BorderConfig with validation
- `crates/puzzle-core/src/seed.rs` - FNV-1a hash_seed, create_rng for ChaCha8Rng
- `crates/puzzle-core/src/edge.rs` - TabDirection, Edge, EdgeParams types
- `crates/puzzle-core/src/connector.rs` - ConnectorGenerator trait, NullConnector stub
- `crates/puzzle-core/src/lib.rs` - Module declarations and pub use re-exports

## Decisions Made
- Used FNV-1a hash for string-to-u64 seed conversion (portable, not std DefaultHasher which varies across Rust versions)
- Added rand with default-features=false to avoid getrandom panic on wasm32-unknown-unknown
- Added serde_json as dev-dependency only for serialization round-trip tests
- RNG passed as &mut param to ConnectorGenerator::generate() so grid engine controls deterministic sequence

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed rand 0.10 RngExt import for random_bool/random**
- **Found during:** Task 1 (seed tests)
- **Issue:** rand 0.10 moved `random_bool` and `random` to `RngExt` trait, not just `Rng`
- **Fix:** Changed `use rand::Rng` to `use rand::RngExt` in test module
- **Files modified:** crates/puzzle-core/src/seed.rs
- **Verification:** All seed tests pass
- **Committed in:** 4c685d4 (Task 1 commit)

**2. [Rule 3 - Blocking] Added serde_json dev-dependency for serialization tests**
- **Found during:** Task 2 (edge serialization tests)
- **Issue:** Edge/TabDirection serialization tests use serde_json which wasn't a dependency of puzzle-core
- **Fix:** Added `serde_json = "1.0"` as dev-dependency in Cargo.toml
- **Files modified:** crates/puzzle-core/Cargo.toml
- **Verification:** All tests compile and pass
- **Committed in:** 5e4cad7 (Task 2 commit)

**3. [Rule 3 - Blocking] Added module declarations to lib.rs during Task 1**
- **Found during:** Task 1 (seed test verification)
- **Issue:** config.rs and seed.rs tests couldn't run without being declared as modules in lib.rs
- **Fix:** Added `pub mod config; pub mod seed;` to lib.rs early (Task 2 completed the full wiring)
- **Files modified:** crates/puzzle-core/src/lib.rs
- **Verification:** Seed and config tests pass
- **Committed in:** 4c685d4 (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (1 bug, 2 blocking)
**Impact on plan:** All auto-fixes necessary for compilation and testing. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All foundation types ready for Plan 02 (Grid Layout Engine)
- ConnectorGenerator trait ready for Plan 03 (Classic Knob Connector) implementation
- PuzzleConfig provides validated parameters for grid construction
- Existing puzzle-wasm crate remains backward-compatible

## Self-Check: PASSED

- All 6 created/modified files verified on disk
- Both task commits (4c685d4, 5e4cad7) verified in git history
- 42/42 tests passing
- puzzle-wasm compiles without changes

---
*Phase: 02-grid-engine-data-model*
*Completed: 2026-03-02*

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

# Phase 2 Plan 3: WASM Integration Summary

**generate_grid WASM endpoint wiring PuzzleConfig JSON through PuzzleGrid engine to GridResponse JSON, completing the Phase 2 config→engine→boundary vertical slice**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-02T23:58:07Z
- **Completed:** 2026-03-03T00:00:26Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- generate_grid WASM endpoint accepts full PuzzleConfig JSON and returns structured GridResponse
- GridResponse includes piece breakdown, edge summary, and per-piece info with border flags
- Same seed produces identical output (determinism verified)
- Existing compute_pieces endpoint backward compatible
- Full workspace: 74 tests passing (65 puzzle-core + 9 puzzle-wasm)
- WASM binary builds cleanly at 119KB raw / 56.5KB gzipped, no getrandom panics

## Task Commits

Each task was committed atomically:

1. **Task 1: Create generate_grid WASM endpoint** - `35eb9ba` (feat)
2. **Task 2: End-to-end build verification** - No commit (verification-only task, all tests from Task 1)

**Plan metadata:** (pending)

## Files Created/Modified
- `crates/puzzle-wasm/src/lib.rs` - Added generate_grid endpoint with GridResponse, PieceInfo, PieceBreakdownInfo, EdgeSummary types and 9 tests
- `crates/puzzle-wasm/Cargo.toml` - Added serde dependency for WASM response type serialization
- `Cargo.lock` - Updated lockfile with serde for puzzle-wasm

## Decisions Made
- Created WASM-specific response types (GridResponse, PieceInfo) rather than serializing core types directly — keeps WASM API surface intentional
- Empty seed defaults to "default" since WASM cannot use getrandom; documented as Phase 4 responsibility to pass JS-generated random seeds
- Response excludes bezier edge geometry (not yet generated) — Phase 3 will add connector data

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 2 complete: all 3 plans executed (types, engine, WASM boundary)
- Full vertical slice proven: JSON config → Rust grid engine → JSON grid data via WASM
- Ready for Phase 3 (Connector Generation & SVG Export) which will populate edge.connector fields and add SVG output
- WASM binary size reasonable (56.5KB gzipped) with room for connector generation code

## Self-Check: PASSED

- All 3 modified files verified on disk
- Task commit (35eb9ba) verified in git history
- 74/74 tests passing (65 puzzle-core + 9 puzzle-wasm)
- WASM binary builds cleanly (119KB raw / 56.5KB gzipped)

---
*Phase: 02-grid-engine-data-model*
*Completed: 2026-03-03*
