# S03 — Custom Border UI — Research

**Date:** 2026-03-21

## Summary

S03 is straightforward UI wiring. The WASM backend (S02) already supports `border_shape` in `PuzzleConfig` — passing `"heart"` or `"star"` through `generate_edges_binary()` returns boundary-aware edge data, border path commands, and a cached SVG with the shape contour. The Canvas renderer (`drawBorder`, `drawVisibleEdges`) already handles the CMD_* protocol for arbitrary border paths including cubic beziers. What's missing is: (1) a `<select>` dropdown in the HTML for shape selection, (2) wiring it into `buildConfig()` to pass `border_shape` through to WASM, (3) URL param persistence for `border_shape`, and (4) fixing the piece count display which currently hardcodes `rows * cols` but needs the actual included count for boundary puzzles.

The one non-trivial concern is piece count: `generate_edges_binary()` currently returns `{ edges, border, width, height }` with no piece count. For boundary puzzles the JS `rows * cols` calculation is wrong. The fix is to add a `piece_count` field to the WASM response so JS can display the correct count.

## Recommendation

Add a border shape dropdown to the controls panel, wire it through `buildConfig()` → `generate_edges_binary()`, persist in URL params, and extend the WASM response with `piece_count` for accurate display. This is three small tasks: WASM response extension, HTML/JS UI wiring, and URL param sync.

## Implementation Landscape

### Key Files

- `web/index.html` — Add a `<select id="border-shape">` dropdown in a new control section (between Dimensions and Parameters, or as its own section). Options: "Rectangle" (default, no border_shape), "Heart", "Star".
- `web/src/main.ts` — Five changes needed:
  1. Add `borderShapeSelect` DOM reference
  2. Update `buildConfig()` to include `border_shape` when not "none"/"rectangle"
  3. Update `generatePuzzle()` to use WASM-returned piece count instead of computing `rows * cols`
  4. Update `loadFromURL()` to restore border shape from URL param
  5. Update `updateURL()` to persist border shape to URL param
  6. Wire `change` event on the dropdown to `scheduleGenerate()`
- `crates/puzzle-wasm/src/lib.rs` — Extend `generate_edges_binary()` to include `piece_count` (and optionally a breakdown) in the returned JS object. When `border_shape` is set, compute count from `BoundaryPuzzle::included_cell_count()`. When absent, count is `rows * cols`.
- `web/src/style.css` — No changes expected; the existing `.control-section`, `.input-group`, and `select` styles cover the new dropdown.

### Build Order

1. **T01: Extend WASM response with piece count** — Add `piece_count` to `generate_edges_binary()` return object. This is the only Rust change. Quick — 3 lines of code in the WASM layer. Verify with `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml`.

2. **T02: Add border shape UI + wiring** — HTML dropdown, `buildConfig()` integration, event wiring, piece count display fix, URL param sync. Pure frontend. Verify by running `npm run dev:wasm && npm run dev` in `web/` and visually confirming heart/star shapes render in Canvas, piece count is correct, URL params persist.

### Verification Approach

1. `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — all existing + new WASM tests pass
2. `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — WASM still compiles
3. Build WASM for web: `cd web && npm run dev:wasm` — succeeds
4. Manual browser test: select "Heart" from dropdown → Canvas shows heart-shaped puzzle with curved border, fewer pieces than rectangular. Select "Star" → Canvas shows star-shaped puzzle. Select "Rectangle" → reverts to normal rectangular puzzle.
5. Piece count display: heart should show fewer than rows×cols pieces; rectangular should show exact rows×cols.
6. URL param roundtrip: select heart shape → URL contains `border=heart` → reload page → heart is still selected and puzzle renders correctly.
7. Download SVG: with heart selected → downloaded SVG opens in browser/Inkscape showing heart-shaped puzzle outline.

## Constraints

- `generate_edges_binary` returns a raw `JsValue` (not JSON), using `js_sys::Object` and `js_sys::Reflect::set`. Adding `piece_count` follows the same pattern as `width`/`height`.
- The `borderData` and `edgesData` already drive Canvas rendering for boundary puzzles — no Canvas code changes needed.
- The existing `drawBorder()` function handles moveTo (0), lineTo (1), curveTo (2), closePath (3) commands — this already renders heart curves and star line segments correctly.
- `buildConfig()` must omit `border_shape` entirely (not send `null`) when rectangular is selected, to maintain backward compatibility with serde defaults (`#[serde(default)]`).

## Common Pitfalls

- **Piece count display assumes rectangular grid** — The JS code computes `rows * cols`, `4` corners, `2*(rows-2) + 2*(cols-2)` edges, `(rows-2)*(cols-2)` interior. For boundary puzzles these formulas are all wrong. The simplest fix is showing just the total from WASM (e.g. "32 pieces inside heart shape") rather than trying to compute breakdown categories which don't apply cleanly to boundary puzzles.
- **URL param `border` vs `border_shape`** — use a short param name like `border` in the URL (not `border_shape`) to keep URLs compact, but map it to the config key `border_shape` in `buildConfig()`.
- **Default select value must map to "no border_shape in config"** — Use value `""` or `"none"` for the rectangular option and only add `border_shape` to the config object when a real shape is selected.
