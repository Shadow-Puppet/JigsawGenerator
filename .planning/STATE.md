---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: completed
stopped_at: Completed 04-02-PLAN.md
last_updated: "2026-03-03T23:01:15.697Z"
progress:
  total_phases: 4
  completed_phases: 4
  total_plans: 8
  completed_plans: 8
---

# Project State: Puzzle Pattern Generator

## Project Reference

**Core Value:** Generate geometrically valid, laser-cuttable jigsaw puzzle SVG patterns with configurable dimensions and procedural variation so no two puzzles are identical.

**Current Focus:** All phases complete. Full puzzle generator with GUI, URL sharing, SVG download ready.

## Current Position

**Phase:** 04-web-gui-live-preview
**Plan:** 2 of 2
**Status:** Milestone complete

```
Phase 1 [x] Build Pipeline & WASM Foundation (1/1 plans)
Phase 2 [x] Grid Engine & Data Model (3/3 plans)
Phase 3 [x] Connector Generation & SVG Export (2/2 plans)
Phase 4 [x] Web GUI & Live Preview (2/2 plans)
```

## Performance Metrics

| Metric | Value |
|--------|-------|
| Phases complete | 4/4 |
| Plans complete | 8/8 |
| Tasks complete | 18/18 |
| Requirements met | 18/18 |
| 01-01 duration | 6 min |
| 02-01 duration | 4 min |
| 02-02 duration | 3 min |
| 02-03 duration | 2 min |
| 03-01 duration | 5 min |
| 03-02 duration | 6 min |
| 04-01 duration | 2 min |
| 04-02 duration | 5 min |

## Accumulated Context

### Key Decisions
- Rust + WASM for core generation, vanilla TypeScript + Vite for GUI
- `kurbo` for 2D geometry, `rand_chacha` for deterministic seeded RNG
- Shared-edge data model (adjacent pieces reference same path data) — must be designed from Phase 2
- Connector trait abstraction from Phase 2 even with single connector type
- SVG strict subset for laser cutter compatibility (absolute coords, inline attributes, physical units)
- JSON serialization for WASM boundary — simple, debuggable, flexible
- vite-plugin-wasm for zero-config WASM loading in Vite
- Installed rustup locally for wasm32-unknown-unknown target (Arch Linux system Rust)
- FNV-1a hash for string-to-u64 seed conversion (portable, not std DefaultHasher)
- rand with default-features=false to avoid getrandom panic on wasm32-unknown-unknown
- RNG passed as &mut param to ConnectorGenerator (grid controls deterministic sequence)
- Shared-edge model with index references: pieces reference edges by index into h_edges/v_edges, not by value
- Fixed RNG consumption order: h_edges row-major then v_edges row-major for seed determinism
- WASM response types (GridResponse, PieceInfo) separate from puzzle-core types — intentional API surface
- Empty seed defaults to "default" in WASM layer; JS generates random seeds in Phase 4
- Separate RNG for connector generation (seed suffix '-connectors') preserves grid construction determinism
- 5 cubic bezier segments per knob: baseline→neck, neck→body, top, body→neck, neck→baseline
    - Neck width 75% of body width creates visible narrowing for snap-fit
- Single <path> element for all cut lines — border closed subpath + internal edge open subpaths
- Kerf compensation removed entirely — broken feature deleted from codebase (quick-009)
- kurbo::Arc for quarter-circle rounded corners → cubic bezier approximation
- Affine transform (translate * rotate) for edge-local to global coordinate mapping
- No debounce on parameter changes — WASM generate_svg fast enough for instant regeneration
- buildConfig() pattern: centralized DOM-to-PuzzleConfig JSON builder reads all inputs
- history.replaceState (not pushState) for URL sync — avoids polluting browser history
- CSS stroke-width override for screen display; downloaded SVGs preserve hairline strokes for laser cutting
- URL param abbreviations: w/h, mm/in, tab/jitter as integer percentages for compact shareable URLs
- Taper range adjusted to 0.30..=1.10; old URL params clamped for backward compat (quick-001)
- Taper slider normalized to 0-1 user-facing range with linear interpolation to internal 0.57-1.32 (quick-002, updated quick-016)
- Kerf feature removed entirely — offset algorithm never worked correctly (quick-009, supersedes quick-008)
- Max tab size capped at 25% (was 45%) to prevent oversized/overlapping connectors (quick-003)
- safe_tab_max() must clamp inputs before validation — otherwise out-of-range slider values cause validation failure, silently preventing max updates (quick-004)
- Optional size_pct_max/taper_max fields with serde skip_serializing_if; randomize helpers consume zero RNG when None for backward compat (quick-005)
- convertDimensions() with factor 25.4 and parseFloat(toFixed(2)) for clean unit conversion display (quick-006)
- SVG viewBox normalization: remove width/height attrs, rely on viewBox for responsive fill; re-generate from WASM on download to preserve physical dimensions (quick-007)
- Cursor-centered zoom via transform-origin 0,0 with translate+scale; zoom resets on puzzle regeneration (quick-007)
- Piece count input auto-calcs best rows/cols for target with squarest-piece tiebreaker; no URL param needed (derived from rows*cols) (quick-010)
- Piece size warning at <10mm threshold, visible but non-blocking (quick-010)
- cross_length field in EdgeParams; knob base = min(length, cross_length) for uniform sizing across axes (quick-011)
- safe_tab_max floor removed — no .max(0.15), prevents forced overlap at extreme aspect ratios (quick-011)
- enforceConstraints(source) replaces checkPieceSize(); auto-adjusts unlocked section, shows warnings when locked (quick-012)
- Lock/unlock toggles on Grid Size and Dimensions headers; unlocked = auto-adjust, locked = warning mode (quick-012)
- rAF throttle on generatePuzzle() for all rapid-fire input handlers; direct call kept for unit-select and initial load (quick-013)
- Cached SVG path element avoids DOM querySelector per zoom/pan frame (quick-013)
- Inline JS piece count math replaces compute_pieces WASM roundtrip (quick-013)
- GPU compositing via will-change:transform on #svg-container; CSS containment on #svg-viewport (quick-014)
- Inline JS tab max math replaces safe_tab_max WASM roundtrip — identical formula, zero overhead (quick-014)
- SVG path attribute diffing: subsequent renders update d/viewBox attrs only, skip innerHTML (quick-014)
- rAF-throttled pan/zoom transforms via scheduleTransform(); direct call kept for button clicks (quick-014)
- URL sync debounced at 300ms trailing to prevent replaceState spam during rapid input (quick-014)
- WASM -O3 instead of -Os for speed over size; LTO + codegen-units=1 for max release optimization (quick-014)
- Canvas 2D replaces SVG for puzzle display; context transform (not CSS transform) for crisp rendering at any zoom (quick-015)
- Binary edge data transfer: 36-float fixed stride per edge, zero string parsing; command-prefixed border encoding (quick-015)
- AABB viewport culling with 35% margin for knob protrusion — only visible edges drawn per frame (quick-015)
- SVG cached at generation time via thread_local! in WASM; get_cached_svg() for instant download without regeneration (quick-015)

### Research Flags
- **Phase 3 (Connectors):** Complete. Connector generation + SVG export pipeline fully functional.
- **Phase 4 (GUI):** Standard patterns, no research needed.

### Learnings
- Arch Linux system Rust doesn't include wasm32-unknown-unknown; need rustup for WASM targets
- WASM release build with wasm-opt produces ~56KB gzipped with grid engine (was ~48KB with minimal logic)
- rand 0.10: `random_bool`/`random`/`random_range` are on `RngExt` trait, not just `Rng`
- kurbo `bounding_box()` requires importing `ParamCurveExtrema` trait
- kurbo BezPath.to_svg() outputs absolute uppercase commands (M, L, C, Z) — perfect for laser cutter SVG
- WASM binary with full SVG export pipeline is ~93KB gzipped (up from ~56KB with grid engine only)
- WASM binary with binary export + js-sys is ~78KB gzipped / 169KB uncompressed (quick-015)

### TODOs
(None yet)

### Blockers
(None)

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 001 | Adjust taper range - make .30 the minimum and increase maximum by 10% | 2026-03-04 | 90325ae | [001-adjust-taper-range-make-30-the-minimum-a](./quick/001-adjust-taper-range-make-30-the-minimum-a/) |
| 002 | Change taper range to 0-1 (user) / 0.5-1.2 (internal) | 2026-03-04 | 31e6755 | [002-change-taper-range-to-0-5-1-2-internally](./quick/002-change-taper-range-to-0-5-1-2-internally/) |
| 003 | Set the max tab size to 25% | 2026-03-04 | ec0f42a | [3-set-the-max-tab-size-to-25](./quick/3-set-the-max-tab-size-to-25/) |
| 004 | Rebuild WASM and fix safe_tab_max validation chicken-and-egg bug | 2026-03-04 | 2e1ce8e | [4-rebuild-wasm-and-fix-safe-tab-max-valida](./quick/4-rebuild-wasm-and-fix-safe-tab-max-valida/) |
| 005 | Add randomize-per-edge option to tab size and taper | 2026-03-04 | 2f589ef | [5-add-randomize-per-edge-option-to-tab-siz](./quick/5-add-randomize-per-edge-option-to-tab-siz/) |
| 006 | Auto-convert width/height when unit dropdown changes | 2026-03-04 | c1a459d | [6-convert-dimension-values-automatically-w](./quick/6-convert-dimension-values-automatically-w/) |
| 007 | SVG preview: fill container width, add ruler, enable zoom/pan | 2026-03-04 | 5ee5e4d | [7-svg-preview-fill-container-width-add-rul](./quick/7-svg-preview-fill-container-width-add-rul/) |
| 008 | Fix kerf width setting - offset border only, not connectors | 2026-03-04 | b2d77de | [8-investigate-and-fix-kerf-width-setting-c](./quick/8-investigate-and-fix-kerf-width-setting-c/) |
| 009 | Remove kerf width feature entirely | 2026-03-04 | efb9242 | [9-remove-kerf-width-feature-entirely-delet](./quick/9-remove-kerf-width-feature-entirely-delet/) |
| 010 | Add piece count input with auto row/col calculation | 2026-03-04 | c559db3 | [10-add-piece-count-input-with-auto-row-col-](./quick/10-add-piece-count-input-with-auto-row-col-/) |
| 011 | Fix vertical axis knob scaling to match horizontal | 2026-03-05 | d69e29e | [11-fix-vertical-axis-knob-scaling-to-match-](./quick/11-fix-vertical-axis-knob-scaling-to-match-/) |
| 012 | Auto-adjust dimensions/grid to prevent warnings with lock/unlock toggles | 2026-03-05 | fac44e9 | [12-auto-adjust-dimensions-grid-to-prevent-w](./quick/12-auto-adjust-dimensions-grid-to-prevent-w/) |
| 013 | Optimize SVG rendering performance for large puzzles | 2026-03-05 | 3f25219 | [13-optimize-svg-rendering-performance-for-l](./quick/13-optimize-svg-rendering-performance-for-l/) |
| 014 | Buttery smooth UI: GPU compositing, inline tab max, SVG diffing, rAF throttle, URL debounce, WASM -O3/LTO | 2026-03-07 | 600936d | [14-buttery-smooth-ui-gpu-compositing-inline](./quick/14-buttery-smooth-ui-gpu-compositing-inline/) |
| 015 | Canvas 2D renderer with viewport culling and binary WASM data transfer | 2026-03-07 | 8157208 | [15-optimize-large-puzzle-performance-for-sm](./quick/15-optimize-large-puzzle-performance-for-sm/) |
| 016 | Adjust taper parameter range: 0.50..=1.20 to 0.57..=1.32 | 2026-03-09 | 666518c | [16-adjust-taper-parameter-range-min-stays-0](./quick/16-adjust-taper-parameter-range-min-stays-0/) |

## Session Continuity

**Last session:** 2026-03-09T13:33:00Z
**Stopped at:** Completed quick task 016: Adjust taper range from 0.50..=1.20 to 0.57..=1.32
**Next action:** Milestone complete — all 4 phases done, quick-001 through 016 applied

---
Last activity: 2026-03-09 - Completed quick task 016: Taper range adjusted to 0.57..=1.32
*Last updated: 2026-03-09T13:33:00Z*
