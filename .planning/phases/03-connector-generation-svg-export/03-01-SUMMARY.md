---
phase: 03-connector-generation-svg-export
plan: 01
subsystem: core
tags: [bezier, kurbo, connector, jigsaw, procedural-generation, tdd]

# Dependency graph
requires:
  - phase: 02-grid-engine-data-model
    provides: ConnectorGenerator trait, Edge struct with connector field, PuzzleGrid with shared-edge model
provides:
  - ClassicKnobConnector implementing ConnectorGenerator trait
  - PuzzleGrid.generate_connectors() method populating all internal edges
  - 5-segment cubic bezier knob shapes with visible neck for snap-fit
affects: [03-connector-generation-svg-export, 04-web-gui-live-preview]

# Tech tracking
tech-stack:
  added: []
  patterns: [edge-local-connector-coordinates, separate-rng-for-connectors, named-proportion-constants]

key-files:
  created:
    - crates/puzzle-core/src/classic_connector.rs
  modified:
    - crates/puzzle-core/src/grid.rs
    - crates/puzzle-core/src/lib.rs

key-decisions:
  - "Separate RNG for connector generation (seed suffix '-connectors') to avoid disturbing grid construction RNG sequence"
  - "5 cubic bezier segments per knob: baseline→neck, neck→body, top, body→neck, neck→baseline"
  - "Neck width 75% of body width creates visible narrowing for snap-fit"
  - "Knob height = 1.2x knob width for Ravensburger-style proportions"

patterns-established:
  - "Named constants for all knob proportion ratios (KNOB_HEIGHT_RATIO, NECK_WIDTH_RATIO, etc.)"
  - "TDD red-green-refactor cycle for core algorithm implementation"

requirements-completed: [CONN-01, CONN-02]

# Metrics
duration: 5min
completed: 2026-03-03
---

# Phase 3 Plan 1: ClassicKnobConnector Summary

**5-segment cubic bezier knob connector with visible neck, procedural jitter variation, and grid wiring via TDD**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-03T03:03:02Z
- **Completed:** 2026-03-03T03:08:47Z
- **Tasks:** 3 (TDD: RED → GREEN → REFACTOR)
- **Files modified:** 3

## Accomplishments
- ClassicKnobConnector produces traditional Ravensburger-style knob shapes with 5 cubic bezier segments
- Visible neck narrowing (75% of body width) ensures laser-cut pieces snap together
- Procedural jitter varies both control point positions and knob center offset per edge
- TabDirection::Out = +Y knob, TabDirection::In = -Y knob (correct edge-local convention)
- PuzzleGrid.generate_connectors() populates all internal edges; borders remain None
- Same seed deterministically produces identical connectors across runs
- validate() checks origin, endpoint, curve continuity, and bounding box bounds

## Task Commits

Each task was committed atomically (TDD cycle):

1. **Task 1: RED - Write failing tests** - `c75a282` (test)
2. **Task 2: GREEN - Implement ClassicKnobConnector** - `78cae75` (feat)
3. **Task 3: REFACTOR - Clean up constants and edge cases** - `27dac38` (refactor)

## Files Created/Modified
- `crates/puzzle-core/src/classic_connector.rs` - ClassicKnobConnector struct implementing ConnectorGenerator trait with 5-segment bezier knob generation and validation
- `crates/puzzle-core/src/grid.rs` - Added generate_connectors() method and 3 new grid-level connector tests
- `crates/puzzle-core/src/lib.rs` - Added classic_connector module export

## Decisions Made
- Used separate RNG (seeded with `"{seed}-connectors"`) for connector generation to avoid disturbing the grid construction RNG sequence — ensures adding/changing connectors doesn't change tab directions
- 5 cubic bezier segments per knob matching the research anatomy: baseline→neck entry, neck→body widening, rounded top, body→neck narrowing, neck exit→baseline
- Extracted all knob proportion magic numbers to named constants for maintainability

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- ClassicKnobConnector ready for SVG export pipeline (Plan 02)
- All internal edges populated with bezier curves for path construction
- Edge-local coordinates ready for Affine transform to global space

---
*Phase: 03-connector-generation-svg-export*
*Completed: 2026-03-03*
