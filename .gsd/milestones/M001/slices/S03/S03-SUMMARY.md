---
id: S03
parent: M001
milestone: M001
provides:
  - ClassicKnobConnector implementing ConnectorGenerator trait
  - PuzzleGrid.generate_connectors() method populating all internal edges
  - 5-segment cubic bezier knob shapes with visible neck for snap-fit
  - generate_svg() function producing laser-cutter-ready SVG from PuzzleGrid
  - offset_path() kerf compensation for polyline offset
  - kerf_width field on PuzzleConfig
  - generate_svg WASM endpoint for browser-based SVG generation
requires: []
affects: []
key_files: []
key_decisions:
  - Separate RNG for connector generation (seed suffix '-connectors') to avoid disturbing grid construction RNG sequence
  - 5 cubic bezier segments per knob: baseline→neck, neck→body, top, body→neck, neck→baseline
  - Neck width 75% of body width creates visible narrowing for snap-fit
  - Knob height = 1.2x knob width for Ravensburger-style proportions
  - Single <path> element with all cut lines — border as closed subpath, internal edges as open subpaths
  - Kerf compensation via polyline offset with miter/bevel joins — no re-smoothing for v1
  - kurbo::Arc for quarter-circle rounded corners converted to cubic beziers
  - Affine transform (translate * rotate) for edge-local to global coordinate mapping
patterns_established:
  - Named constants for all knob proportion ratios (KNOB_HEIGHT_RATIO, NECK_WIDTH_RATIO, etc.)
  - TDD red-green-refactor cycle for core algorithm implementation
  - SVG export: generate_svg(grid) → build_puzzle_path() → offset_path() → build_svg_document()
  - Edge transform: Affine::translate(start) * Affine::rotate(angle) maps (0,0)-(length,0) to global coords
  - Kerf offset: flatten curves to polylines, compute outward normals, miter join at vertices
observability_surfaces: []
drill_down_paths: []
duration: 6min
verification_result: passed
completed_at: 2026-03-03
blocker_discovered: false
---
# S03: Connector Generation Svg Export

**# Phase 3 Plan 1: ClassicKnobConnector Summary**

## What Happened

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

# Phase 3 Plan 2: SVG Export Pipeline Summary

**Laser-cutter-ready SVG export with single-path construction, rounded border corners, edge-local-to-global bezier transforms, polyline kerf compensation, and WASM endpoint**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-03T03:11:45Z
- **Completed:** 2026-03-03T03:18:32Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- SVG export pipeline produces complete, laser-cutter-compatible SVG with mm dimensions, viewBox, hairline black stroke, absolute coordinates, and single `<path>` element
- Border rendered as closed subpath with straight lines and quarter-circle rounded corners at all 4 puzzle corners using kurbo::Arc → cubic bezier conversion
- Internal edges transformed from edge-local coordinates to global coordinates via Affine (translate + rotate), emitted as open subpaths with cubic bezier curves
- Kerf compensation offsets all paths outward by half the kerf width using polyline offset with miter/bevel joins
- WASM endpoint `generate_svg()` accepts PuzzleConfig JSON and returns complete SVG string, backward compatible with JSON missing kerf_width
- 113 tests passing across workspace (98 puzzle-core + 15 puzzle-wasm), WASM binary ~93KB gzipped

## Task Commits

Each task was committed atomically:

1. **Task 1: Add kerf_width to PuzzleConfig and create SVG export + kerf modules** - `7e7ed0a` (feat)
2. **Task 2: Add generate_svg WASM endpoint and end-to-end verification** - `85b106a` (feat)

## Files Created/Modified
- `crates/puzzle-core/src/svg_export.rs` - SVG path construction (build_puzzle_path, edge_transform) and document generation (build_svg_document, generate_svg)
- `crates/puzzle-core/src/kerf.rs` - Polyline offset for kerf compensation (offset_path) with miter/bevel joins
- `crates/puzzle-core/src/config.rs` - Added kerf_width field with validation, serde(default), and from_input parameter
- `crates/puzzle-core/src/grid.rs` - Updated test_config helper for kerf_width field
- `crates/puzzle-core/src/lib.rs` - Added svg_export and kerf module exports
- `crates/puzzle-wasm/src/lib.rs` - Added generate_svg WASM endpoint with 6 new tests

## Decisions Made
- Used single `<path>` element for all cut lines (border + internal edges) — laser cutter software handles subpaths within one path element
- Kerf compensation implemented as polyline offset (flatten → offset → miter join) rather than curve offset — simpler, sufficient for v1 laser cutting precision
- Quarter-circle corners via kurbo::Arc with 0.01mm tolerance cubic bezier approximation — indistinguishable from true arcs at laser cutter resolution
- Edge transform uses Affine::translate(start.to_vec2()) * Affine::rotate(angle) — maps connector curves from edge-local (0,0)-(length,0) space to global puzzle coordinates

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 3 complete: connector generation + SVG export pipeline fully functional
- WASM endpoint ready for Phase 4 web GUI integration
- generate_svg() accepts PuzzleConfig JSON, returns complete SVG for download/preview
- Ready for Phase 4: Web GUI & Live Preview

---
*Phase: 03-connector-generation-svg-export*
*Completed: 2026-03-03*
