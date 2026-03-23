# S04: Whimsy Drag-Drop & Grid Adaptation

**Goal:** User drags a heart/star shape onto the puzzle canvas, positions and resizes it freely, and the grid adapts with the whimsy boundary as a smooth cut line — no tabs at the boundary.
**Demo:** Place a heart whimsy on a 6×8 puzzle → grid cells under the heart are removed, piece count updates, heart outline appears as a cut line. Resize the whimsy → grid re-adapts. Reload the page → whimsy position/shape/scale restored from URL. Download SVG → whimsy contour included.

## Must-Haves

- Whimsy config fields (`whimsy_shape`, `whimsy_x`, `whimsy_y`, `whimsy_scale`) in `PuzzleConfig` with backward-compatible serde defaults
- All three WASM endpoints (`generate_svg`, `generate_edges_binary`, `generate_grid`) use `BoundaryPuzzle::new_with_hole()` when whimsy params are present
- Hole shape contour included in both SVG export and binary border data (for Canvas drawing and laser cutting)
- All four config combinations handled: neither border nor whimsy, border only, whimsy only, both
- Canvas drag-drop for placing whimsy shape on puzzle (mousedown/mousemove/mouseup)
- Resize handles or scroll-to-resize on the placed whimsy
- Whimsy overlay drawn instantly on Canvas during drag (no WASM call for visual feedback)
- Debounced WASM regeneration during drag, immediate on drop
- URL params (`ws`, `wx`, `wy`, `wsc`) persist whimsy state across reloads
- Only one whimsy at a time (R012) — placing new replaces old
- Whimsy position stored in puzzle mm coordinates, not screen pixels (zoom/pan independent)

## Proof Level

- This slice proves: integration (Rust WASM + Canvas UI + real-time interaction)
- Real runtime required: yes (browser with Canvas and WASM)
- Human/UAT required: yes (visual verification of drag-drop responsiveness, R011)

## Verification

- `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` — all boundary tests pass including new hole-export tests
- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — all WASM tests pass including new whimsy endpoint tests
- `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — WASM compiles
- `grep -q 'whimsy_shape' crates/puzzle-core/src/config.rs` — config fields exist
- `grep -q 'resolve_whimsy_shape' crates/puzzle-wasm/src/lib.rs` — WASM whimsy helper exists
- `grep -q 'whimsy' web/src/main.ts` — JS whimsy wiring exists
- `grep -q 'whimsy' web/index.html` — whimsy UI controls exist
- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- whimsy_invalid` — invalid whimsy shape returns structured error JSON
- Browser: drag heart onto puzzle → grid cells removed, piece count decreases, heart outline visible as cut line
- Browser: resize whimsy → grid adapts, piece count changes
- Browser: reload page → whimsy restored from URL params
- Browser: download SVG → whimsy outline present in file

## Observability / Diagnostics

- Runtime signals: `console.warn` when `piece_count` missing from WASM response (existing), WASM error JSON for invalid whimsy shape
- Inspection surfaces: browser console shows whimsy state variables; URL params encode full whimsy config; WASM `generate_grid()` response includes piece count reflecting hole
- Failure visibility: WASM returns `{"error":"Unknown border shape: '...'"}` for invalid whimsy shapes; Canvas overlay draws without WASM call so drag remains responsive even if generation fails
- Redaction constraints: none

## Integration Closure

- Upstream surfaces consumed: `BoundaryPuzzle::new_with_hole()` from `boundary.rs` (S02), `heart_path()`/`star_path()` from `shapes.rs` (S01), `resolve_border_shape()` pattern from WASM layer (S02), `buildConfig()`/`scheduleGenerate()`/`drawPuzzle()` patterns from `main.ts` (S03)
- New wiring introduced in this slice: whimsy config fields in `PuzzleConfig`, `resolve_whimsy_shape()` in WASM layer, whimsy drag-drop interaction on Canvas, whimsy overlay drawing in `drawPuzzle()`, URL param persistence for whimsy state
- What remains before the milestone is truly usable end-to-end: S05 (whimsy sub-puzzle splitting), S06 (export polish and integrated SVG)

## Tasks

- [ ] **T01: Wire whimsy config into WASM endpoints with hole contour export** `est:45m`
  - Why: The WASM layer must accept whimsy parameters and produce hole-aware output before any JS can generate puzzles with whimsy. The hole shape contour must also appear in SVG/binary export so it renders as a cut line on Canvas and in laser-cut SVG.
  - Files: `crates/puzzle-core/src/config.rs`, `crates/puzzle-core/src/boundary.rs`, `crates/puzzle-wasm/src/lib.rs`
  - Do: (1) Add `whimsy_shape`, `whimsy_x`, `whimsy_y`, `whimsy_scale` to `PuzzleConfig` with `#[serde(default)]`. (2) Add `hole: Option<BezPath>` field to `BoundaryPuzzle`, set in `new_with_hole()`, left as `None` in `new()`. (3) Update `generate_boundary_svg()` and `boundary_border_to_binary()` to include the hole contour when `self.hole` is `Some`. (4) Add `resolve_whimsy_shape()` helper in WASM that takes shape name + x/y/scale and returns a translated/scaled BezPath. (5) Update all three WASM endpoints to handle the four config combinations: neither, border only, whimsy only, both. (6) Add Rust tests for whimsy config, hole export, and WASM endpoints with whimsy.
  - Verify: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- boundary` passes; `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` passes; `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` passes
  - Done when: WASM endpoints accept whimsy config, produce fewer pieces with whimsy, SVG includes hole contour, binary border data includes hole contour, backward compatibility preserved

- [ ] **T02: Build whimsy drag-drop Canvas overlay with resize, URL persistence, and generation wiring** `est:1h30m`
  - Why: The user-facing interaction — drag-drop placement, resize, real-time grid adaptation, and URL persistence — is the core of this slice and satisfies R005, R006, R011, R012.
  - Files: `web/src/main.ts`, `web/index.html`, `web/src/style.css`
  - Do: (1) Add whimsy UI controls to `index.html` — a "Whimsy Shape" section with a dropdown (None/Heart/Star) and a Remove Whimsy button. (2) Add whimsy state variables in `main.ts` (`whimsyShape`, `whimsyX`, `whimsyY`, `whimsyScale`, `isDraggingWhimsy`, `isResizingWhimsy`). (3) Implement drag-drop: selecting a whimsy shape from dropdown starts placement mode; click on canvas places it at that position (converted to mm coords via inverse zoom/pan transform); subsequent mousedown on an existing whimsy starts drag; mousemove updates position; mouseup finalizes. (4) Implement resize: scroll wheel while hovering over whimsy adjusts `whimsyScale`; or drag corner handles. (5) Draw whimsy overlay in `drawPuzzle()` — draw the shape outline using the existing `drawBorder()` CMD pattern but with a distinct stroke color/style; draw immediately during drag without waiting for WASM. (6) Wire whimsy config into `buildConfig()` — include `whimsy_shape`, `whimsy_x`, `whimsy_y`, `whimsy_scale` only when whimsy is active (matching D025 optional field pattern). (7) Debounce WASM regeneration during drag using `scheduleGenerate()` (existing rAF throttle), immediate `generatePuzzle()` on mouseup. (8) URL params: save/restore `ws`, `wx`, `wy`, `wsc` in `updateURL()`/`loadFromURL()`. (9) Update piece count display to show whimsy-aware count. (10) Remove whimsy button clears state and regenerates. (11) One whimsy at a time — selecting a new shape replaces the old. Guard against drag when `puzzleWidth === 0`. Relevant skills: `frontend-design`, `make-interfaces-feel-better`.
  - Verify: `grep -q 'whimsy' web/src/main.ts` and `grep -q 'whimsy' web/index.html`; browser visual verification — drag heart onto puzzle, resize, reload preserves position, download SVG contains whimsy outline
  - Done when: User can select a whimsy shape, click to place it on the canvas, drag to reposition, scroll to resize, grid adapts in real-time, URL persists state, download includes whimsy geometry

## Files Likely Touched

- `crates/puzzle-core/src/config.rs` — add whimsy config fields
- `crates/puzzle-core/src/boundary.rs` — add hole field, update exports to include hole contour
- `crates/puzzle-wasm/src/lib.rs` — add `resolve_whimsy_shape()`, update all 3 endpoints
- `web/src/main.ts` — whimsy state, drag-drop, resize, overlay drawing, config wiring, URL params
- `web/index.html` — whimsy UI controls (dropdown, remove button)
- `web/src/style.css` — whimsy control styles, cursor states
