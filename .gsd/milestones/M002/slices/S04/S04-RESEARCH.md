# S04: Whimsy Drag-Drop & Grid Adaptation — Research

**Date:** 2026-03-21
**Status:** Ready for planning

## Summary

S04 adds whimsy piece placement: the user drags a shape (heart/star) onto the puzzle canvas, positions and resizes it freely, and the grid adapts in real-time — removing cells inside the whimsy and using the whimsy boundary as the cut line. This slice has two distinct halves: (1) **Rust/WASM backend** — extend `PuzzleConfig` with whimsy parameters and wire `BoundaryPuzzle::new_with_hole()` into all three WASM endpoints, and (2) **TypeScript frontend** — build drag-drop interaction, resize handles, Canvas rendering of the whimsy outline, and real-time regeneration.

The Rust side is straightforward — `new_with_hole()` already exists and is tested. The WASM layer needs new config fields and a whimsy-aware code path in `generate_edges_binary()`. The frontend is the bulk of the work: drag-drop state management, coordinate transforms (screen↔puzzle mm), Canvas overlay drawing, resize handles, debounced regeneration, and URL persistence of whimsy state. There is no existing drag-drop or interactive placement code in `main.ts` — it must be built from scratch, but the patterns for Canvas drawing, zoom/pan coordinate transforms, and config wiring are well-established.

## Recommendation

**Approach:** Split into three tasks — (1) Rust/WASM config + generation with whimsy hole, (2) Whimsy Canvas overlay (drawing the shape, drag-drop, resize), (3) Integration wiring (real-time regeneration, URL params, piece count update, SVG download).

**Why this order:** The WASM endpoint must accept whimsy parameters before the JS can generate puzzles with holes. The Canvas overlay (drawing and interaction) is the most complex task and should be isolated. Integration wiring connects them.

**Key design decisions:**
- Whimsy state lives in JS, sent to WASM as config fields (`whimsy_shape`, `whimsy_x`, `whimsy_y`, `whimsy_scale`). The WASM layer resolves the shape, applies scale/translate, and passes it as the `hole` to `BoundaryPuzzle::new_with_hole()`.
- Drag interactions work in puzzle mm coordinates (not screen pixels), using the inverse of the existing zoom/pan transform. This ensures the whimsy position is zoom/pan-independent.
- Debounce regeneration during drag at ~60fps using the existing `requestAnimationFrame` throttle pattern (`scheduleGenerate`), with the whimsy outline drawn as a Canvas overlay that updates instantly (no WASM call needed for visual feedback).
- Only one whimsy at a time (R012) — placing a new one replaces the old one.

## Implementation Landscape

### Key Files

- `crates/puzzle-core/src/config.rs` — Add `whimsy_shape: Option<String>`, `whimsy_x: Option<f64>`, `whimsy_y: Option<f64>`, `whimsy_scale: Option<f64>` fields to `PuzzleConfig` with `#[serde(default)]`
- `crates/puzzle-core/src/boundary.rs` — Already has `BoundaryPuzzle::new_with_hole()` — no changes needed
- `crates/puzzle-core/src/shapes.rs` — Already has `heart_path()` and `star_path()` — no changes needed
- `crates/puzzle-wasm/src/lib.rs` — Add `resolve_whimsy_shape()` helper (scale + translate a shape BezPath), update `generate_edges_binary()`, `generate_svg()`, and `generate_grid()` to use `new_with_hole()` when whimsy params are present. Same pattern as border_shape handling.
- `web/src/main.ts` — Major additions: whimsy state variables, drag-drop event handling on Canvas, resize handles, whimsy overlay drawing in `drawPuzzle()`, config wiring in `buildConfig()`, URL param persistence. ~300-400 new lines.
- `web/index.html` — Add whimsy shape selector UI (dropdown or draggable palette) and optional controls (remove whimsy button, scale display)
- `web/src/style.css` — Styles for whimsy controls, cursor states during drag

### Build Order

**Task 1: WASM whimsy endpoint** — Prove the WASM layer accepts whimsy config, creates a hole in the grid, and returns correct binary/SVG data with fewer pieces. This unblocks all JS work. Verification: `cargo test` with whimsy config produces fewer pieces than without; SVG output contains the correct boundary.

**Task 2: Canvas whimsy overlay + drag-drop** — Build the interactive layer: draw whimsy shape outline on Canvas, handle mousedown/mousemove/mouseup for drag placement, handle resize via corner handles or scroll/pinch on the shape. This is the most complex task and the primary risk for R011 (responsiveness). Verification: visual — shape appears, drags smoothly, resizes, coordinates persist across zoom/pan.

**Task 3: Integration wiring** — Connect drag-drop state to WASM generation: `buildConfig()` includes whimsy params, `generatePuzzle()` produces hole-aware output, piece count updates, URL params `ws/wx/wy/wsc` persist whimsy state, SVG download includes whimsy filename suffix. Verification: end-to-end — drag shape onto puzzle, grid updates, reload preserves position, download includes all geometry.

### Verification Approach

**Rust tests:**
- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — existing + new whimsy tests
- New test: `test_generate_svg_with_whimsy` — config with whimsy params produces SVG with hole
- New test: `test_generate_grid_with_whimsy_fewer_pieces` — whimsy reduces piece count
- New test: `test_whimsy_config_backward_compat` — config without whimsy fields works

**Browser verification:**
- Drag heart onto puzzle → grid cells under heart removed, heart outline visible as cut line
- Resize whimsy → grid adapts, more/fewer cells removed
- Zoom/pan → whimsy stays in correct puzzle-space position
- Reload page → whimsy position/shape/scale restored from URL
- Download SVG → whimsy outline in SVG file

**Structural checks:**
- `grep 'whimsy_shape' crates/puzzle-core/src/config.rs` — config field exists
- `grep 'whimsy' crates/puzzle-wasm/src/lib.rs` — WASM handling exists
- `grep 'whimsy' web/src/main.ts` — JS wiring exists

## Constraints

- **No grid snap (D016)** — whimsy position is free-form in mm coordinates, not snapped to cell boundaries
- **No tabs on whimsy boundary (D017)** — the whimsy contour itself is the cut line; edges adjacent to the hole are simply excluded, not trimmed with connectors
- **One whimsy at a time (D018/R012)** — placing a new whimsy replaces the old
- **PuzzleGrid doesn't implement Clone (K006)** — in the WASM layer, must extract whimsy config fields before consuming the grid, same pattern as border_shape extraction
- **Shape resolution in WASM layer (K008)** — whimsy shape name → BezPath resolution stays in `resolve_border_shape()` / new `resolve_whimsy_shape()`, not in JS
- **Existing `scheduleGenerate` uses rAF** — whimsy drag events should use this same throttle for WASM regeneration, with a separate immediate overlay draw for visual feedback

## Common Pitfalls

- **Coordinate system confusion** — The Canvas uses screen pixels with zoom/pan transform. Whimsy position must be stored in puzzle mm coords (pre-transform), converted to/from screen coords using the existing `baseScale * zoomLevel` + panX/panY transform. The inverse transform is: `mmX = (screenX - panX) / (baseScale * zoomLevel)`, `mmY = (screenY - panY) / (baseScale * zoomLevel)`.
- **Whimsy position outside puzzle bounds** — User could drag the whimsy partially outside the puzzle area. This is fine geometrically (the hole simply doesn't overlap with any grid cells in the out-of-bounds area), but the Canvas drawing should clip or allow it naturally.
- **Empty config field pattern (D025/K009)** — Whimsy config fields must be omitted from `buildConfig()` when no whimsy is placed, not sent as empty/zero values. Follow the `border_shape` pattern: only include when truthy.
- **Border + whimsy interaction** — When both `border_shape` and whimsy are active, need `BoundaryPuzzle::new_with_hole(grid, border_boundary, whimsy_hole)`. When only whimsy (no custom border), the boundary is the full rectangular grid and only the hole applies. The WASM code needs to handle all four combinations: neither, border only, whimsy only, both.

## Open Risks

- **Interactive performance (R011)** — WASM boolean ops + grid generation must complete fast enough for responsive drag. The existing `scheduleGenerate` (rAF throttle) limits to 60fps, but if WASM generation takes >16ms, the preview will lag. Mitigation: draw the whimsy outline immediately as an overlay, debounce WASM regeneration separately (e.g., 100ms debounce during active drag, immediate on drop). Measure actual generation time — for a 6×8 grid, it's likely <5ms.
- **Whimsy drawing while no grid is generated yet** — If user starts dragging before first generation, the Canvas coordinate system may not be initialized (`puzzleWidth === 0`). Need to guard against this.
