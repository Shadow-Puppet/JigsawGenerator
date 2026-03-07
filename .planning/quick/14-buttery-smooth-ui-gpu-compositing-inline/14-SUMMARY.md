---
phase: quick-14
plan: 01
subsystem: web-ui, wasm-build
tags: [performance, gpu-compositing, svg-diffing, raf-throttle, lto]
dependency_graph:
  requires: []
  provides: [gpu-composited-pan-zoom, svg-path-diffing, inline-tab-max, debounced-url-sync, wasm-o3-lto]
  affects: [web/src/style.css, web/src/main.ts, crates/puzzle-wasm/Cargo.toml, Cargo.toml, web/tsconfig.json]
tech_stack:
  added: []
  patterns: [will-change-transform, css-containment, raf-throttle, svg-attr-diffing, url-debounce, wasm-lto]
key_files:
  created: []
  modified:
    - web/src/style.css
    - web/src/main.ts
    - web/tsconfig.json
    - crates/puzzle-wasm/Cargo.toml
    - Cargo.toml
decisions:
  - "Inline JS tab max math replaces safe_tab_max WASM roundtrip — identical formula, zero overhead"
  - "SVG path diffing uses regex d='...' extraction from WASM SVG string for subsequent renders"
  - "scheduleTransform() rAF-throttles only continuous handlers (wheel, mousemove, touchmove); button clicks stay direct"
  - "URL sync debounced at 300ms trailing — prevents replaceState spam during rapid input"
  - "wasm-opt -O3 instead of -Os — optimize for speed over size for generation throughput"
  - "LTO + codegen-units=1 for maximum cross-crate optimization in release builds"
metrics:
  duration: "4 min"
  completed: "2026-03-07"
  tasks_completed: 2
  tasks_total: 2
---

# Quick Task 14: Buttery Smooth UI — GPU Compositing & Inline Optimizations

**One-liner:** GPU-composited pan/zoom with CSS containment, inline JS tab-max math, SVG path attribute diffing, rAF-throttled transforms, debounced URL sync, and WASM -O3/LTO build.

## Task Results

| # | Task | Commit | Key Changes |
|---|------|--------|-------------|
| 1 | CSS GPU compositing + JS performance optimizations | `5854902` | will-change:transform, contain:layout style paint, inline tab max, SVG path diffing, rAF throttle, URL debounce |
| 2 | WASM -O3 + LTO build optimization | `600936d` | wasm-opt -O3, lto=true, codegen-units=1 |

## Detailed Changes

### Task 1: CSS GPU compositing + JS performance optimizations

**CSS:**
- Added `will-change: transform` to `#svg-container` — promotes element to GPU layer for hardware-accelerated zoom/pan transforms
- Added `contain: layout style paint` to `#svg-viewport` — isolates SVG containment so layout changes don't propagate up the DOM tree

**JS — Inline tab max math:**
- Replaced `updateTabMax()` which called `safe_tab_max()` WASM function with pure JS equivalent
- Same formula: computes maxH, maxV, maxApproach from grid dimensions, takes 0.9 * min(), caps at 0.25
- Removed `safe_tab_max` from WASM import (kept `init`, `generate_svg`, `init_panic_hook`)

**JS — SVG path attribute diffing:**
- Added module-level `svgEl: SVGSVGElement | null` to track cached SVG element
- First render: full innerHTML + normalize (remove width/height, ensure viewBox) + cache svgEl and cachedSvgPath
- Subsequent renders: extract `d` and `viewBox` attrs from WASM SVG string via regex, update attributes only
- Updated `resetZoom()` to use cached `svgEl` instead of `svgContainer.querySelector("svg")`

**JS — rAF-throttled transforms:**
- Added `scheduleTransform()` with `transformRafPending` flag to coalesce rapid transform updates
- Applied to: wheel zoom, mousemove pan, touchmove single-finger pan, touchmove pinch zoom
- Kept direct `applyTransform()` for: resetZoom, zoomIn/zoomOut button clicks

**JS — Debounced URL sync:**
- Added `scheduleURLUpdate()` with 300ms trailing debounce via `setTimeout`
- Replaced `updateURL()` call in `generatePuzzle()` with `scheduleURLUpdate()`

### Task 2: WASM -O3 + LTO build optimization

- Changed `wasm-opt` from `["-Os"]` to `["-O3"]` in crates/puzzle-wasm/Cargo.toml — optimizes for speed instead of size
- Added `[profile.release]` to root Cargo.toml with `lto = true` and `codegen-units = 1` — enables link-time optimization and single codegen unit for maximum cross-crate optimization
- Rebuilt WASM binary successfully with new settings (161KB, 76KB gzipped)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added tsconfig.json paths for puzzle-wasm module resolution**
- **Found during:** Task 1 verification
- **Issue:** `npx tsc --noEmit` could not find module 'puzzle-wasm' — the Vite alias in vite.config.ts resolves it at build time but tsc doesn't read Vite config
- **Fix:** Added `"paths": { "puzzle-wasm": ["../crates/puzzle-wasm/pkg"] }` to web/tsconfig.json
- **Files modified:** web/tsconfig.json
- **Commit:** 5854902

## Verification

- `npx tsc --noEmit` in web/ — passes clean
- `npx vite build` in web/ — succeeds (16.85KB JS, 6.52KB CSS, 161.55KB WASM)
- `ls web/pkg/puzzle_wasm_bg.wasm` — exists (160,126 bytes)
- WASM rebuilt with -O3 + LTO settings

## Self-Check: PASSED

All files exist, all commits verified, all content assertions pass.
