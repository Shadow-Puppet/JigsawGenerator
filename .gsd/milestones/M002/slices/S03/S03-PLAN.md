# S03: Custom Border UI

**Goal:** User selects a border shape from a dropdown in the web UI, Canvas preview shows the non-rectangular puzzle, piece count is accurate, and the selection persists in URL params.
**Demo:** Select "Heart" from the border shape dropdown → Canvas renders a heart-shaped puzzle with fewer pieces than rectangular → piece count display shows the correct number → URL contains `border=heart` → reload → heart is still selected and renders correctly → Download SVG produces heart-shaped puzzle.

## Must-Haves

- Border shape `<select>` dropdown with Rectangle (default), Heart, Star options
- `buildConfig()` includes `border_shape` when a non-rectangular shape is selected
- Piece count display uses WASM-returned `piece_count` instead of JS-computed `rows * cols`
- URL param `border` persists shape selection across page reloads
- `generate_edges_binary()` WASM response includes `piece_count` field
- Canvas rendering works for all three options (Rectangle, Heart, Star)

## Proof Level

- This slice proves: integration
- Real runtime required: yes (browser + WASM)
- Human/UAT required: yes (visual confirmation of Canvas rendering)

## Observability / Diagnostics

- **piece_count in WASM response:** The `generate_edges_binary()` JS object now includes `piece_count` (f64). The frontend can display this value and compare it against `rows * cols` to confirm boundary filtering is active. If `piece_count == rows * cols` when a non-rectangular border is selected, the boundary computation may have failed silently.
- **Error shape:** When `border_shape` is invalid, `generate_edges_binary()` returns `{ error: "Unknown border shape: '...'" }`. The frontend should check for the `error` property and surface it to the user.
- **Inspection surfaces:** In the browser console, `JSON.parse(result)` after a WASM call shows piece_count alongside edges/border/width/height. The `generate_grid()` JSON endpoint also includes `piece_breakdown.total` for cross-verification.
- **Failure visibility:** If `piece_count` is missing from the WASM response, the JS frontend falls back to `rows * cols` (incorrect for boundary puzzles). This silent fallback should be logged as a warning in the browser console.

## Verification

- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — all existing + new WASM tests pass
- `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` — WASM compiles
- `grep -q 'piece_count' crates/puzzle-wasm/src/lib.rs` — piece_count field exists in WASM response
- `grep -q 'border-shape' web/index.html` — border shape dropdown exists in HTML
- `grep -q 'border_shape' web/src/main.ts` — border shape wiring exists in JS
- `grep -q 'border' web/src/main.ts | grep -q 'URLSearchParams\|params'` — URL param sync exists
- Build succeeds: `cd web && npm run dev:wasm` (WASM compiles to web target)
- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- test_border_shape_invalid_returns_error` — invalid border shape returns error JSON (failure path)

## Integration Closure

- Upstream surfaces consumed: `crates/puzzle-wasm/src/lib.rs` (`generate_edges_binary()` with `border_shape` support from S02), `crates/puzzle-core/src/boundary.rs` (`BoundaryPuzzle::included_cell_count()`)
- New wiring introduced in this slice: HTML dropdown → `buildConfig()` → WASM `generate_edges_binary()` → Canvas render; URL param `border` ↔ dropdown state sync
- What remains before the milestone is truly usable end-to-end: S04 (whimsy drag-drop), S05 (sub-puzzle), S06 (export polish)

## Tasks

- [x] **T01: Add piece_count to WASM generate_edges_binary response** `est:15m`
  - Why: The JS currently computes `rows * cols` for piece count, which is wrong for boundary puzzles. The WASM layer already knows the real count from BoundaryPuzzle filtering. Adding `piece_count` to the response gives the frontend accurate data.
  - Files: `crates/puzzle-wasm/src/lib.rs`
  - Do: Add `piece_count` field (u32) to the JS object returned by `generate_edges_binary()`. For boundary puzzles, compute from `BoundaryPuzzle::included_cell_count()`. For rectangular puzzles, use `rows * cols`. Add a test that verifies heart border returns `piece_count < rows * cols` and rectangular returns `piece_count == rows * cols`.
  - Verify: `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` passes all tests; `cargo check --manifest-path crates/puzzle-wasm/Cargo.toml --target wasm32-unknown-unknown` compiles
  - Done when: `generate_edges_binary()` returns a JS object with `piece_count` field for both rectangular and boundary puzzles, and tests confirm the count is correct

- [ ] **T02: Add border shape dropdown, wire config/URL, fix piece count display** `est:30m`
  - Why: This is the user-facing feature — a dropdown to select border shape, wiring through to WASM, correct piece count display, and URL persistence. Without this, the WASM border support from S02 is invisible to users.
  - Files: `web/index.html`, `web/src/main.ts`
  - Do: (1) Add `<select id="border-shape">` dropdown in HTML with Rectangle/Heart/Star options. (2) Add `borderShapeSelect` DOM reference and wire `change` event to `scheduleGenerate()`. (3) Update `buildConfig()` to include `border_shape` when not rectangular. (4) Update `generatePuzzle()` to use `result.piece_count` from WASM instead of JS-computed breakdown. (5) Update `loadFromURL()` to restore border shape from `border` URL param. (6) Update `updateURL()` to persist `border` param. (7) Include border in download filename when shape is active.
  - Verify: `grep -q 'border-shape' web/index.html && grep -q 'border_shape' web/src/main.ts && grep -q 'piece_count' web/src/main.ts`
  - Done when: Border shape dropdown appears in the UI, selecting Heart/Star passes `border_shape` through to WASM, piece count displays WASM-returned count, URL param `border` persists selection across reloads

## Files Likely Touched

- `crates/puzzle-wasm/src/lib.rs`
- `web/index.html`
- `web/src/main.ts`
