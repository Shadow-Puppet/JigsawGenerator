---
id: T01
parent: S02
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
requires: []
affects: []
key_files: []
key_decisions: []
patterns_established: []
observability_surfaces: []
drill_down_paths: []
duration: 4min
verification_result: passed
completed_at: 2026-03-02
blocker_discovered: false
---
# T01: 02-grid-engine-data-model 01

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
