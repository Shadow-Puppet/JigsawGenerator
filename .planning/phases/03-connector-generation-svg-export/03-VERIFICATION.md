---
phase: 03-connector-generation-svg-export
verified: 2026-03-02T22:45:00Z
status: passed
score: 11/11 must-haves verified
gaps: []
human_verification:
  - test: "Open generated SVG in LightBurn or Inkscape to verify no parse errors"
    expected: "SVG opens cleanly with visible knob connectors on internal edges, rounded corners on border, hairline stroke"
    why_human: "Programmatic checks verify SVG structure but not actual rendering in laser cutter software"
  - test: "Visually inspect knob shapes for aesthetic quality"
    expected: "Classic Ravensburger-style knobs with visible neck narrowing, proportional to edge length"
    why_human: "Shape aesthetics are subjective and can't be verified by code"
  - test: "Laser cut a test piece with kerf compensation and verify snug fit"
    expected: "Pieces with kerf_width > 0 produce tighter interlocking fit than kerf_width = 0"
    why_human: "Physical fit quality requires real-world cutting and testing"
---

# Phase 3: Connector Generation & SVG Export Verification Report

**Phase Goal:** The engine generates complete jigsaw puzzles with classic knob connectors and exports production-ready SVG files that work in laser cutter software
**Verified:** 2026-03-02T22:45:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | ClassicKnobConnector produces cubic bezier curves forming a traditional knob shape | ✓ VERIFIED | `classic_connector.rs` (487 lines): `impl ConnectorGenerator for ClassicKnobConnector` generates 5 `CubicBez` segments with neck, body, and rounded top. 11 tests validate shape geometry. |
| 2 | Each edge has procedural variation from jitter (different control point positions, different knob center offset) | ✓ VERIFIED | 4 independent `cp_jitter` values + `center_jitter` applied per edge. `test_jitter_produces_variation` confirms different RNG seeds produce different control points. |
| 3 | TabDirection::Out produces knob in +Y direction, TabDirection::In produces knob in -Y direction | ✓ VERIFIED | `dir_sign` = +1.0 for Out, -1.0 for In, applied to `knob_h`. Tests `test_direction_out_positive_y` and `test_direction_in_negative_y` confirm sign conventions. |
| 4 | Same seed produces identical connector curves; different seeds produce different curves | ✓ VERIFIED | Separate RNG via `create_rng("{seed}-connectors")`. `test_generate_connectors_deterministic` and `test_zero_jitter_deterministic` confirm reproducibility. |
| 5 | PuzzleGrid.generate_connectors() populates all internal edge connector fields | ✓ VERIFIED | `generate_connectors()` iterates h_edges and v_edges, sets `connector = Some(curves)` for non-border edges. `test_generate_connectors_populates_internal_edges` verifies all internal edges have `Some`, all borders remain `None`. |
| 6 | Generated SVG contains a single path element with all cut lines, shared edges appearing exactly once | ✓ VERIFIED | `build_puzzle_path()` constructs one BezPath, `build_svg_document()` emits single `<path>`. Edges iterated from shared arrays (not per-piece). `test_svg_contains_path_element` confirms. |
| 7 | SVG has explicit mm dimensions, matching viewBox, absolute coordinates, hairline black stroke | ✓ VERIFIED | `build_svg_document()` outputs `width='{w}mm'`, `viewBox='0 0 {w} {h}'`, `stroke='#000000'`, `stroke-width='0.001mm'`, `fill='none'`. 5 tests verify each attribute. `test_svg_no_relative_commands` confirms absolute-only commands. |
| 8 | Border edges are straight lines with quarter-circle rounded corners at the 4 puzzle corners | ✓ VERIFIED | `build_puzzle_path()` uses `line_to` for border segments, `append_quarter_arc()` via `kurbo::Arc` at 4 corners. `test_svg_border_is_closed` confirms closed subpath with `Z`. |
| 9 | Internal edges render as connector bezier curves transformed to global coordinates | ✓ VERIFIED | `edge_transform()` computes `Affine::translate * Affine::rotate`, applied to all curve control points. `test_svg_contains_cubic_curves` and `test_svg_internal_edges_present` confirm `C` commands and correct M-command count (18 for 3x4 grid). |
| 10 | Kerf compensation offsets all paths outward by half the kerf width when kerf > 0 | ✓ VERIFIED | `kerf.rs` (280 lines): `offset_path()` flattens to polylines, computes outward normals, offsets by `kerf_width / 2.0` with miter/bevel joins. Called from `generate_svg()` when `kerf_width > 0`. Tests verify offset outward + path structure preserved. |
| 11 | WASM endpoint generate_svg() returns complete SVG string from PuzzleConfig JSON | ✓ VERIFIED | `puzzle-wasm/src/lib.rs` lines 219-242: `#[wasm_bindgen] pub fn generate_svg()` deserializes config, creates grid, calls `generate_connectors()`, returns `puzzle_core::generate_svg(&grid)`. 6 WASM tests pass. Backward compatible via `#[serde(default)]` on `kerf_width`. |

**Score:** 11/11 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/puzzle-core/src/classic_connector.rs` | ClassicKnobConnector implementing ConnectorGenerator trait, min 80 lines | ✓ VERIFIED | 487 lines, `impl ConnectorGenerator for ClassicKnobConnector` at line 67, 5-segment bezier generation, 11 named constants, 11 tests |
| `crates/puzzle-core/src/grid.rs` | generate_connectors() method on PuzzleGrid | ✓ VERIFIED | `fn generate_connectors()` at line 142, iterates h_edges/v_edges, populates internal edge connectors, 3 connector-specific tests |
| `crates/puzzle-core/src/svg_export.rs` | SVG path construction and document generation, min 100 lines, exports generate_svg | ✓ VERIFIED | 369 lines, `pub fn generate_svg()`, `build_puzzle_path()`, `edge_transform()`, `build_svg_document()`, `append_quarter_arc()`, 12 tests |
| `crates/puzzle-core/src/kerf.rs` | Polyline offset for kerf compensation, min 40 lines, exports offset_path | ✓ VERIFIED | 280 lines, `pub fn offset_path()`, miter/bevel join logic, 4 tests |
| `crates/puzzle-core/src/config.rs` | kerf_width field added to PuzzleConfig | ✓ VERIFIED | `pub kerf_width: f64` with `#[serde(default)]` at line 140, validation 0.0..=1.0, 3 kerf-specific tests |
| `crates/puzzle-wasm/src/lib.rs` | generate_svg WASM endpoint | ✓ VERIFIED | `#[wasm_bindgen] pub fn generate_svg()` at line 219, 6 WASM-specific tests |
| `crates/puzzle-core/src/lib.rs` | Module exports for classic_connector, svg_export, kerf | ✓ VERIFIED | `pub mod classic_connector;`, `pub mod kerf;`, `pub mod svg_export;` with `pub use` re-exports |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `classic_connector.rs` | `connector.rs` | `impl ConnectorGenerator for ClassicKnobConnector` | ✓ WIRED | Line 67: implements both `generate()` and `validate()` methods |
| `grid.rs` | `classic_connector.rs` | calls `generate()` on ConnectorGenerator | ✓ WIRED | Lines 159, 174: `connector.generate(&params, &mut rng)` called for each non-border edge |
| `svg_export.rs` | `grid.rs` | reads PuzzleGrid h_edges/v_edges | ✓ WIRED | Lines 98, 119: iterates `grid.h_edge(row, col)` and `grid.v_edge(row, col)` |
| `svg_export.rs` | `kerf.rs` | calls offset_path when kerf > 0 | ✓ WIRED | Line 25: `path = offset_path(&path, grid.config.kerf_width)` |
| `puzzle-wasm/lib.rs` | `svg_export.rs` | calls generate_svg from WASM endpoint | ✓ WIRED | Line 241: `puzzle_core::generate_svg(&grid)` |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| CONN-01 | 03-01-PLAN | Puzzle generates classic knob connector shapes using cubic bezier curves | ✓ SATISFIED | ClassicKnobConnector produces 5-segment cubic bezier knobs. 11 shape tests pass. |
| CONN-02 | 03-01-PLAN | Each edge is procedurally varied (random direction, control point jitter) | ✓ SATISFIED | 4 independent cp_jitter values + center_jitter per edge. Tab directions randomly assigned in grid.rs. |
| EXPT-01 | 03-02-PLAN | User can export puzzle as SVG with laser-cutter compatible strokes | ✓ SATISFIED | generate_svg() produces SVG with mm units, viewBox, hairline black stroke, absolute coords, single path element. WASM endpoint exposes this to browser. |
| EXPT-02 | 03-02-PLAN | User can apply kerf compensation to adjust path offsets for snug piece fit | ✓ SATISFIED | kerf_width config field, offset_path() polyline offset, integrated into SVG pipeline. Tests confirm offset produces different (larger) paths. |

**No orphaned requirements.** REQUIREMENTS.md maps CONN-01, CONN-02, EXPT-01, EXPT-02 to Phase 3. All 4 are claimed by plans and satisfied.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| — | — | No TODOs, FIXMEs, placeholders, or stub implementations found | — | — |

**No anti-patterns detected.** All phase 3 files are clean. Match-arm catch-alls (`_ => {}`) in kerf.rs and grid.rs are valid Rust patterns for exhaustive matching, not stubs.

### Build & Test Results

| Check | Result | Details |
|-------|--------|---------|
| `cargo test --workspace` | ✅ All pass | 98 puzzle-core + 15 puzzle-wasm = 113 total tests |
| `cargo build -p puzzle-core` | ✅ Clean | No warnings |
| WASM build | ⚠️ Env limitation | `wasm32-unknown-unknown` target not installed in verification environment. WASM code compiles and passes all 15 native tests. Not a code issue. |

### Human Verification Required

### 1. SVG Compatibility with Laser Cutter Software

**Test:** Open a generated SVG file in LightBurn, Inkscape, or equivalent laser cutter software.
**Expected:** SVG opens without parse errors. Visible knob connectors on internal edges, rounded corners on border, single cut path with hairline black stroke.
**Why human:** Programmatic checks verify SVG structure (attributes, path commands) but cannot confirm actual rendering behavior in third-party software.

### 2. Knob Shape Aesthetic Quality

**Test:** Visually inspect the connector shapes at various tab_size and jitter settings.
**Expected:** Classic Ravensburger-style knobs with visible neck narrowing, proportional dimensions, smooth curves. Different seeds produce visibly different but all aesthetically pleasing shapes.
**Why human:** Shape aesthetics are subjective and require visual judgment.

### 3. Physical Kerf Compensation Fit

**Test:** Laser cut a puzzle with kerf_width = 0 and again with kerf_width = 0.1, compare piece fit.
**Expected:** Pieces cut with kerf compensation fit more snugly together than those without.
**Why human:** Physical fit quality can only be verified through actual laser cutting and assembly.

### Gaps Summary

No gaps found. All 11 observable truths are verified. All 7 artifacts exist, are substantive (well above minimum line counts), and are properly wired. All 5 key links are connected and functional. All 4 requirements (CONN-01, CONN-02, EXPT-01, EXPT-02) are satisfied. No anti-patterns detected. 113 tests pass across the workspace.

The phase goal — "The engine generates complete jigsaw puzzles with classic knob connectors and exports production-ready SVG files that work in laser cutter software" — is achieved by the codebase.

---

_Verified: 2026-03-02T22:45:00Z_
_Verifier: Claude (gsd-verifier)_
