# 006 — Whimsy sub-puzzle splitting

**What:** A placed whimsy splits into N sub-pieces with working connectors inside its contour, visible in the Canvas preview and included in the SVG export.
**Why:** Makes the whimsy itself a proper puzzle region rather than a single blob. Closes the geometric capability side of M002 before the export-polish slice.

## Must-haves

- `whimsy_sub_pieces: Option<u32>` on `PuzzleConfig` with `#[serde(default)]`.
- Sub-grid dimension computation: derive rows × cols from target N and the whimsy bounding-box aspect ratio, clamped to a ≥ 2×2 minimum.
- Sub-puzzle generation reuses `BoundaryPuzzle::new(sub_grid, whimsy_contour)` with an isolated RNG seed `"{seed}-whimsy-sub"` so changing sub-piece count doesn't perturb the main-puzzle RNG.
- Sub-puzzle binary data appends to the main `Float64Array` (same `EDGE_STRIDE` format) and its contour rides in the same border data as the main hole.
- Sub-puzzle connector paths appended to the main SVG's single `<path>`.
- Coordinate translation from sub-grid local space to puzzle-global mm space (offset by whimsy bounding-box origin).
- UI numeric input (2–16 range), hidden until a whimsy is active.
- URL param `wsp` for sub-piece count.
- Graceful degradation: sub-pieces silently skipped when whimsy is inactive or the whimsy is too small to contain a 2×2 sub-grid. No error.

## Task split

1. **Rust + WASM** (~1h). Add config field, implement `generate_sub_puzzle()` helper, wire into all three endpoints, unit tests for sub-grid dims / cell count / determinism, WASM tests asserting M-command count grows with sub-pieces.
2. **Frontend** (~30m). Numeric input with show/hide, state + URL param, `buildConfig()` inclusion when active, updated piece-count text. No new Canvas drawing code — sub-puzzle edges arrive via the existing `Float64Array` pipeline and render through `drawVisibleEdges()` + `drawBorder()`.

## Verification

- `cargo test -p puzzle-core -- sub_puzzle`, `cargo test -p puzzle-wasm -- sub_piece`, `cargo check --target wasm32-unknown-unknown -p puzzle-wasm`.
- `cargo test -p puzzle-wasm -- sub_pieces_no_whimsy_skips` — graceful degradation with no error.
- Browser: place heart whimsy → set sub-pieces = 4 → see internal cut lines → URL shows `wsp=4` → reload preserves state → download SVG contains sub-puzzle paths.

## Integration points

- Consumes: `resolve_whimsy_shape` (005), `BoundaryPuzzle` (003), `PuzzleGrid` + `generate_connectors()` (001/M001), `EDGE_STRIDE` format (001/M001), whimsy frontend state (005).
- Produces for 007: full-capability output across all four config combinations; extra M-commands per sub-piece that the export-polish slice needs to validate.
