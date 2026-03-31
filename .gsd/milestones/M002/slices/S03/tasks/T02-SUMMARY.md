---
id: T02
parent: S03
milestone: M002
provides:
  - Border shape dropdown in web UI with Rectangle/Heart/Star options
  - buildConfig() conditionally includes border_shape for non-rectangular shapes
  - Piece count display uses WASM-returned piece_count (accurate for boundary puzzles)
  - URL param "border" persists shape selection across reloads
  - Download SVG filename includes border shape name when active
key_files:
  - web/index.html
  - web/src/main.ts
key_decisions:
  - "Empty string value for Rectangle option means buildConfig() omits border_shape entirely — avoids 'Unknown border shape' error from Rust's serde default handling"
  - "Boundary puzzles show simplified piece count format ('N pieces (shape border)') since corner/edge/interior breakdown doesn't apply to non-rectangular shapes"
  - "Added console.warn fallback when piece_count is missing from WASM response — enables diagnosis of stale WASM binaries"
patterns_established:
  - "URL params use short names ('border') while config uses Rust field names ('border_shape') — maintains existing URL param conventions"
observability_surfaces:
  - "Piece count display is now WASM-driven — if boundary puzzle shows same count as rows*cols, boundary computation failed silently"
  - "URL param 'border' presence/absence confirms conditional omission logic"
  - "console.warn emitted when piece_count is missing from WASM response"
  - "Download filename includes shape suffix (e.g. puzzle-6x8-heart-seed-abc.svg) when border is active"
duration: 10m
verification_result: passed
completed_at: 2026-03-24
blocker_discovered: false
---

# T02: Add border shape dropdown, wire config/URL, fix piece count display

**Added border shape dropdown (Rectangle/Heart/Star) to web UI with config wiring, WASM-driven piece count display, URL persistence, and shape-aware download filenames**

## What Happened

Modified two files to wire the border shape feature end-to-end in the web UI:

**`web/index.html`:** Added a new "Border Shape" control section between Dimensions and Parameters with a `<select id="border-shape">` dropdown offering Rectangle (default, value=""), Heart (value="heart"), and Star (value="star").

**`web/src/main.ts`:** Seven changes:
1. Added `borderShapeSelect` DOM reference variable and cached it in `main()`.
2. Updated `buildConfig()` to conditionally add `border_shape` to the config object — only when the dropdown value is non-empty (omitted for Rectangle to avoid Rust serde errors).
3. Replaced the JS-computed piece count display with WASM-returned `result.piece_count`. Boundary puzzles show simplified format ("28 pieces (heart border)"), rectangular puzzles keep detailed breakdown ("48 pieces (4 corner, 20 edge, 24 interior)"). Falls back with `console.warn` if `piece_count` is missing.
4. Updated `loadFromURL()` to restore border shape from the `border` URL param.
5. Updated `updateURL()` to persist the `border` param when a non-rectangular shape is selected.
6. Wired `borderShapeSelect.addEventListener("change", scheduleGenerate)` so selecting a shape immediately regenerates the puzzle.
7. Updated the download filename to include border shape suffix (e.g. `puzzle-6x8-heart-seed-abc.svg`).

## Verification

All task-level and slice-level verification checks pass:
- 5/5 grep checks for HTML dropdown, DOM reference, config wiring, piece_count consumption, and URL param
- 26/26 WASM tests pass (cargo test)
- WASM compiles to wasm32-unknown-unknown target
- TypeScript compiles with zero errors (npx tsc --noEmit)
- wasm-pack build succeeds (npm run dev:wasm)
- Invalid border shape test passes

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `grep -q 'id="border-shape"' web/index.html` | 0 | ✅ pass | <0.01s |
| 2 | `grep -q 'borderShapeSelect' web/src/main.ts` | 0 | ✅ pass | <0.01s |
| 3 | `grep -q 'border_shape' web/src/main.ts` | 0 | ✅ pass | <0.01s |
| 4 | `grep -q 'piece_count' web/src/main.ts` | 0 | ✅ pass | <0.01s |
| 5 | `grep -q '"border"' web/src/main.ts` | 0 | ✅ pass | <0.01s |
| 6 | `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` | 0 | ✅ pass | 1.0s |
| 7 | `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` | 0 | ✅ pass | 0.6s |
| 8 | `grep -q 'piece_count' crates/puzzle-wasm/src/lib.rs` | 0 | ✅ pass | <0.01s |
| 9 | `grep -q 'border-shape' web/index.html` | 0 | ✅ pass | <0.01s |
| 10 | `grep 'border' web/src/main.ts \| grep -q 'params'` | 0 | ✅ pass | <0.01s |
| 11 | `cd web && npm run dev:wasm` | 0 | ✅ pass | 9.6s |
| 12 | `cd web && npx tsc --noEmit` | 0 | ✅ pass | <1s |
| 13 | `cargo test -- border_shape_invalid` | 0 | ✅ pass | <0.01s |

## Diagnostics

- **Piece count display now WASM-driven:** The frontend text is sourced from `result.piece_count` instead of JS-computed `rows * cols`. If `piece_count` is missing (old WASM binary), a `console.warn` is emitted and display falls back to plain `rows * cols` without the breakdown.
- **Border shape in URL:** The `border` URL param is present when a non-rectangular shape is active. Inspect `window.location.search` to confirm. Its absence when Rectangle is selected confirms conditional omission works.
- **Simplified vs detailed piece count:** Boundary puzzles show `"N pieces (shape border)"` format; rectangular puzzles show `"N pieces (C corner, E edge, I interior)"`. If a boundary puzzle shows the detailed breakdown, `borderShapeSelect.value` check is broken.
- **Download filename includes shape:** When downloading SVG with border active, filename includes shape name (e.g., `puzzle-6x8-heart-seed-abc.svg`).

## Deviations

None — implementation followed the task plan exactly.

## Known Issues

None.

## Files Created/Modified

- `web/index.html` — Added Border Shape `<select>` dropdown section between Dimensions and Parameters
- `web/src/main.ts` — Added borderShapeSelect DOM reference, buildConfig() border_shape wiring, WASM piece_count display, URL param sync, change event wiring, download filename with shape suffix
