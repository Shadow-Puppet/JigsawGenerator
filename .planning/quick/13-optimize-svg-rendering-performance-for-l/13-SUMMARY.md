---
phase: quick-13
plan: 01
subsystem: web-gui
tags: [performance, svg, rendering, throttle]
dependency_graph:
  requires: []
  provides: [rAF-throttled-generation, cached-svg-path, inline-piece-count]
  affects: [web/src/main.ts]
tech_stack:
  added: []
  patterns: [requestAnimationFrame-throttle, cached-dom-reference, inline-computation]
key_files:
  modified:
    - web/src/main.ts
decisions:
  - Keep direct generatePuzzle() for unit-select and initial load (single events, not rapid-fire)
  - Replace compute_pieces WASM import with inline JS math (4 corners, edge formula, interior formula)
  - Cache SVG path after each generatePuzzle() call rather than in applyTransform()
metrics:
  duration: 2 min
  completed: "2026-03-05T04:19:19Z"
---

# Quick Task 13: Optimize SVG Rendering Performance for Large Puzzles

**One-liner:** rAF-throttled WASM generation, cached SVG path for zoom/pan, and inline JS piece count replacing redundant WASM roundtrip.

## What Was Done

### Task 1: Add rAF throttle, cache SVG path, inline piece count

**Commit:** `3f25219`

Three targeted optimizations in `web/src/main.ts`:

1. **requestAnimationFrame throttle** — Added `scheduleGenerate()` function with `rafPending` guard. All rapid-fire input handlers (sliders, grid inputs, dimension inputs, seed input, randomize button, randomize toggles) now call `scheduleGenerate()` instead of `generatePuzzle()` directly. This coalesces 60+ input events/sec down to at most 1 WASM call per animation frame.

2. **Cached SVG path element** — Added `cachedSvgPath` module variable, populated after each SVG generation. `applyTransform()` now uses the cached reference instead of `svgContainer.querySelector("svg path")` on every zoom/pan frame.

3. **Inline piece count math** — Replaced `compute_pieces()` WASM call with 4 lines of JS arithmetic (`total = rows * cols`, `corners = 4`, `edges = 2*(rows-2) + 2*(cols-2)`, `interior = (rows-2) * (cols-2)`). Removed `compute_pieces` import and `PieceBreakdown` interface.

**Kept direct `generatePuzzle()`** in:
- `scheduleGenerate()` rAF callback (the actual execution)
- Unit select change handler (single event, not rapid-fire)
- Initial generate on page load (needs immediate render)

## Deviations from Plan

None — plan executed exactly as written.

## Verification

- `npm run build` succeeds with zero TypeScript errors
- All existing functionality preserved (generate, zoom, pan, download, URL sync)
- No remaining references to `compute_pieces` or `PieceBreakdown` in TypeScript source

## Commits

| # | Hash | Message |
|---|------|---------|
| 1 | `3f25219` | perf(quick-13): optimize SVG rendering for large puzzles |

## Self-Check: PASSED
