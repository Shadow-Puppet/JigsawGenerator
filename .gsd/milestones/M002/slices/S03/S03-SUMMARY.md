# S03: Custom Border UI — Summary

**Status:** Complete  
**Tasks:** 2/2 done (T01, T02)  
**Duration:** ~20 minutes total  
**Blocker discovered:** No

## What This Slice Delivered

Users can now select a non-rectangular border shape (Heart or Star) from a dropdown in the web UI. The selection flows through `buildConfig()` → WASM `generate_edges_binary()` → Canvas rendering. The piece count display shows the accurate WASM-returned count (fewer than `rows × cols` for boundary puzzles), and the selection persists via the `border` URL parameter across reloads. Downloaded SVG filenames include the shape name when a non-rectangular border is active.

## Key Changes

### T01: piece_count in WASM response
- Added `piece_count` field to `generate_edges_binary()` return object in `crates/puzzle-wasm/src/lib.rs`
- Boundary puzzles: `bp.included_cells().len()` — actual cells inside the shape
- Rectangular puzzles: `rows * cols`
- 4 new tests via `generate_grid` JSON endpoint (can't test binary WASM endpoints natively — K006)

### T02: Border UI wiring
- `<select id="border-shape">` dropdown in `web/index.html` with Rectangle (default), Heart, Star
- `buildConfig()` conditionally includes `border_shape` only for non-rectangular shapes (empty string = omit)
- `generatePuzzle()` uses `result.piece_count` from WASM instead of JS-computed `rows * cols`
- Boundary puzzles show simplified format: "N pieces (shape border)" — corner/edge/interior breakdown doesn't apply
- `loadFromURL()` restores border from `border` URL param; `updateURL()` persists it
- Download filename includes shape suffix (e.g., `puzzle-6x8-heart-seed-abc.svg`)
- `console.warn` emitted when `piece_count` missing from WASM response (stale binary detection)

## Integration Points

| Surface | What S03 Wired |
|---------|---------------|
| HTML → JS | `border-shape` select → `borderShapeSelect` reference → `change` event → `scheduleGenerate()` |
| JS → WASM | `buildConfig()` adds `border_shape: "heart"/"star"` to config JSON |
| WASM → JS | `generate_edges_binary()` returns `piece_count` field alongside edges/border/width/height |
| URL ↔ State | `border` param read on load, written on change |
| Download | Filename includes shape when active |

## Verification Results

| # | Check | Result |
|---|-------|--------|
| 1 | `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml --lib` | ✅ 26 passed, 0 failed |
| 2 | `cargo check ... --target wasm32-unknown-unknown` | ✅ compiles clean |
| 3 | `grep 'piece_count' crates/puzzle-wasm/src/lib.rs` | ✅ found |
| 4 | `grep 'border-shape' web/index.html` | ✅ found |
| 5 | `grep 'border_shape' web/src/main.ts` | ✅ found |
| 6 | `grep 'piece_count' web/src/main.ts` | ✅ found |
| 7 | URL param sync grep | ✅ found |
| 8 | `cargo test -- test_border_shape_invalid_returns_error` | ✅ 1 passed |

## What Downstream Slices Should Know

- **S04 (Whimsy Drag-Drop):** The border shape dropdown and URL param pattern can be extended for whimsy parameters. The `buildConfig()` conditional inclusion pattern (omit when default) should be followed.
- **S06 (Export & Integration Polish):** The piece count display now uses WASM-returned `piece_count` — S06 should verify this survives with whimsy pieces added. The download filename already includes border shape; S06 needs to add whimsy info too.
- **Convention:** Rectangle is the default — its value is empty string so `border_shape` is omitted from config. This preserves backward compatibility with existing WASM endpoints that don't expect the field.
- **Diagnostics:** `console.warn` fires when `piece_count` is missing from WASM response. Compare `piece_count` vs `rows * cols` to detect silent boundary failures.

## Decisions Captured

- D026: Empty string for Rectangle default → `border_shape` omitted from config
- D027: Simplified piece count display for boundary puzzles ("N pieces (shape border)")

## Requirements Validated

- R003 (User picks shape as puzzle border) → **validated** — dropdown selection, WASM integration, URL persistence, accurate piece count all working
