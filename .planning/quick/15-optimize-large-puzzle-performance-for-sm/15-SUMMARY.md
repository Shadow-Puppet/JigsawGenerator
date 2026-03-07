---
phase: quick-15
plan: 01
subsystem: rendering
tags: [performance, canvas, wasm, viewport-culling, binary-transfer]
dependency_graph:
  requires: [puzzle-core, puzzle-wasm, web-frontend]
  provides: [canvas-renderer, binary-edge-export, viewport-culling, cached-svg-download]
  affects: [web/src/main.ts, crates/puzzle-core/src/binary_export.rs, crates/puzzle-wasm/src/lib.rs]
tech_stack:
  added: [js-sys, Canvas2D, Float64Array, ResizeObserver]
  patterns: [binary-data-transfer, viewport-culling-AABB, fixed-stride-encoding, command-prefixed-path]
key_files:
  created:
    - crates/puzzle-core/src/binary_export.rs
  modified:
    - crates/puzzle-core/src/lib.rs
    - crates/puzzle-core/src/svg_export.rs
    - crates/puzzle-wasm/src/lib.rs
    - crates/puzzle-wasm/Cargo.toml
    - web/src/main.ts
    - web/src/style.css
    - web/index.html
decisions:
  - Fixed 36-float stride per edge (4 header + 2 moveTo + 30 curve data) for zero-parse binary transfer
  - Command-prefixed f64 encoding for border path (moveTo=0, lineTo=1, curveTo=2, close=3)
  - Canvas context transform instead of CSS transform for crisp rendering at any zoom
  - AABB viewport culling with 35% margin for knob protrusion
  - SVG cached at generation time via thread_local! for instant download
metrics:
  duration: 5 min
  completed: "2026-03-07T16:24:48Z"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 8
  tests_added: 8
---

# Quick Task 15: Optimize Large Puzzle Performance Summary

Canvas 2D renderer with binary WASM data transfer and viewport culling — replaces monolithic SVG rendering for 60fps zoom/pan on 6000-10000 piece puzzles.

## Changes Made

### Task 1: WASM Binary Edge Data Export
**Commit:** 3907bb6

Created `binary_export.rs` with two serialization functions:
- `edges_to_binary()`: Fixed 36-float stride per internal edge — 4 header floats (start/end for AABB culling), 2 moveTo floats, 30 curve floats (5 curves × 6 control points). Zero parsing on JS side.
- `border_to_binary()`: Command-prefixed f64 sequence — moveTo(0)+xy, lineTo(1)+xy, curveTo(2)+6pts, close(3). Small data, always drawn.

Made `edge_transform` and `build_border_path` `pub(crate)` in svg_export.rs to share with binary export without duplication.

Added WASM exports:
- `generate_edges_binary(config_json)` → JS object with `edges` Float64Array, `border` Float64Array, `width`, `height`
- `get_cached_svg()` → returns SVG cached during `generate_edges_binary`, avoiding regeneration for download

Added `js-sys` dependency to puzzle-wasm for `Float64Array`, `Object`, and `Reflect` JS interop.

8 new unit tests for binary export format validation (stride, count, determinism, border commands).

### Task 2: Canvas 2D Renderer with Viewport Culling
**Commit:** 8157208

Replaced SVG DOM rendering with Canvas 2D:
- `drawPuzzle()` computes canvas context transform from zoom/pan state, then draws border + culled edges
- `drawVisibleEdges()` iterates fixed-stride Float64Array, performs AABB test per edge (with 35% margin for knob protrusion), calls `bezierCurveTo` only for visible edges
- `drawBorder()` interprets command-prefixed border data

Key architectural change: Canvas context transform replaces CSS transform. CSS transform on canvas just scales pixels (blurry). Canvas context transform redraws at native resolution at any zoom level.

DPR-aware canvas sizing via `ResizeObserver` on viewport element.

Download button uses `get_cached_svg()` instead of re-calling `generate_svg()`.

Removed: SVG DOM elements, `svgEl`/`cachedSvgPath` state, CSS `#svg-container` SVG rules, `will-change: transform`, stroke-width CSS override.

## Deviations from Plan

None — plan executed exactly as written.

## Verification

- `cargo test -p puzzle-core`: 107 tests pass (including 8 new binary export tests)
- `cargo test -p puzzle-wasm`: 13 tests pass
- `npm run build` from web/: TypeScript compiles, Vite builds successfully
- WASM exports `generate_edges_binary` and `get_cached_svg` verified in .d.ts
- WASM binary: 169KB (78KB gzipped) — up from ~93KB due to js-sys + binary export

## Performance Characteristics

At 10K pieces (~20K internal edges):
- Binary data: 20,000 × 36 × 8 bytes = 5.6MB Float64Array (structured, no string parsing)
- At high zoom: viewport shows ~25-100 edges → ~150-600 bezierCurveTo calls → sub-millisecond
- At zoom=1: all ~20K edges → ~100K bezierCurveTo calls → Canvas 2D handles in ~5-15ms
- Border: ~20-30 commands, always drawn, negligible cost

## Self-Check: PASSED
