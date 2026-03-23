# S05 — Whimsy Sub-Puzzle Splitting — Research

**Date:** 2026-03-21
**Depth:** Targeted research — known patterns (BoundaryPuzzle, PuzzleGrid, connector generation) applied inside a new contour. The main question is decomposition strategy, not technology.

## Summary

S05 turns a solid whimsy piece into a mini-puzzle with internal connectors. The core approach is **grid-within-whimsy**: generate a small rectangular PuzzleGrid that covers the whimsy bounding box, then wrap it in a `BoundaryPuzzle` using the whimsy shape as the boundary. This reuses 100% of the existing cell classification, edge filtering, connector generation, and export pipelines. The only new geometry question — how many rows/cols for the inner grid — is solved by computing rows/cols from a user-provided sub-piece count target, fitting an aspect-ratio-matched grid inside the whimsy bounding box.

The work divides into three natural tasks: (1) Rust core logic for sub-puzzle generation with a new `PuzzleConfig` field and WASM endpoint extension, (2) sub-puzzle edge/border data flowing through the binary export to Canvas, and (3) UI for setting sub-piece count with Canvas rendering of internal whimsy cuts.

## Recommendation

**Reuse `BoundaryPuzzle::new(sub_grid, whimsy_path)` with no new geometric engine.**

1. Add `whimsy_sub_pieces: Option<u32>` to `PuzzleConfig` (serde default, same pattern as other whimsy fields).
2. When `whimsy_sub_pieces` is `Some(n)` and `whimsy_shape` is active, compute a small rows×cols grid that yields approximately N cells inside the whimsy contour.
3. Create a `PuzzleGrid` sized to the whimsy bounding box, generate connectors using a whimsy-specific RNG seed (`"{seed}-whimsy-sub"` for determinism isolation), then wrap in `BoundaryPuzzle::new(sub_grid, whimsy_contour)`.
4. Append the sub-puzzle's included edges to the main puzzle's binary edge data and SVG path data.
5. On the JS side, add a numeric input (2–16 range) that appears when a whimsy is active. Sub-puzzle edges flow through the existing `drawVisibleEdges()` and `drawBorder()` — no new Canvas rendering code needed since they use the same binary format.

This approach requires zero new libraries, no new geometric algorithms, and minimal new code (~100 lines Rust, ~30 lines TS).

## Implementation Landscape

### Key Files

- `crates/puzzle-core/src/config.rs` — Add `whimsy_sub_pieces: Option<u32>` field with `#[serde(default)]`. Same pattern as `whimsy_scale`.
- `crates/puzzle-core/src/boundary.rs` — No structural changes needed. `BoundaryPuzzle::new()` already does exactly what sub-puzzle needs. The existing `generate_boundary_svg()` and `boundary_edges_to_binary()` methods produce the right output for the sub-puzzle grid.
- `crates/puzzle-wasm/src/lib.rs` — Add sub-puzzle generation logic inside the whimsy branches of all three endpoints. Extract `whimsy_sub_pieces` from config before `PuzzleGrid::new()` consumes it (K006 pattern). When sub-pieces are requested: compute sub-grid dimensions, create a second `PuzzleGrid` + `BoundaryPuzzle`, append its edge/border data to the main puzzle's output.
- `web/src/main.ts` — Add sub-pieces numeric input wired to `buildConfig()`. Add `whimsy_sub_pieces` to config JSON when active. Append `wsp` URL param. No new drawing code — sub-puzzle edges arrive in the existing Float64Array format.
- `web/index.html` — Add numeric input (or small dropdown) for sub-piece count in the Whimsy Shape section, visible only when a whimsy is active.

### Sub-Grid Dimension Computation

Given target sub-piece count `N` and whimsy bounding box `(w, h)`:
1. Compute aspect ratio `r = w / h`
2. Find `rows` and `cols` such that `rows * cols ≈ N` and `cols / rows ≈ r`
   - `rows = max(2, round(sqrt(N / r)))`, `cols = max(2, round(rows * r))`
   - If `rows * cols` is significantly different from N, adjust one dimension by ±1
3. Create `PuzzleConfig` with these rows/cols, width=whimsy_w, height=whimsy_h
4. RNG seed: `"{original_seed}-whimsy-sub"` — isolated from both main grid RNG and connector RNG

This naturally produces 2–16 pieces inside the whimsy depending on user input. The BoundaryPuzzle cell classification will exclude cells whose centers fall outside the whimsy contour, so the actual piece count may be less than `rows * cols`. This is expected and matches how border shapes work.

### Build Order

1. **T01: Rust config + sub-puzzle generation + tests** — Add config field, implement sub-puzzle grid computation in a helper function, write unit tests proving sub-puzzle edges are generated inside whimsy contour. Tests should verify: (a) sub-puzzle produces >0 included cells, (b) sub-puzzle edges use the correct EDGE_STRIDE format, (c) determinism holds with sub-puzzle config. This is the riskiest task — proves the grid-inside-whimsy approach works.

2. **T02: WASM endpoint integration + binary/SVG output** — Wire sub-puzzle generation into all three WASM endpoints. The `generate_edges_binary` endpoint must return sub-puzzle edges concatenated with main puzzle edges (same Float64Array) and sub-puzzle border commands concatenated with main border commands. The `generate_svg` endpoint must include sub-puzzle connector paths in the SVG `<path>` `d` attribute. Add WASM-level tests.

3. **T03: UI — sub-piece count input + Canvas rendering** — Add numeric input to Whimsy Shape section. Wire to `buildConfig()`. Add `wsp` URL param. No new Canvas drawing code needed — sub-puzzle data arrives in existing binary format. Browser verification: place whimsy, set sub-pieces to 3, verify internal cut lines appear.

### Verification Approach

**Rust unit tests (T01):**
- `test_sub_puzzle_generates_inside_whimsy` — BoundaryPuzzle with heart contour produces >0 included cells and >0 included edges
- `test_sub_puzzle_edge_count_matches_binary` — binary edge data length is consistent with included edge count
- `test_sub_puzzle_deterministic` — same seed + same whimsy + same sub-pieces = identical SVG
- `test_sub_puzzle_grid_dimensions` — dimension computation produces rows*cols ≈ target N

**WASM tests (T02):**
- `test_generate_svg_with_sub_pieces` — SVG output with sub-pieces has more M commands than without
- `test_generate_grid_sub_pieces_in_response` — `piece_count` still reflects main puzzle (not sub-puzzle); sub-piece count returned separately
- `test_sub_pieces_backward_compat` — JSON without `whimsy_sub_pieces` field works unchanged

**Browser verification (T03):**
- Place heart whimsy → set sub-pieces to 4 → verify internal connector lines visible inside the whimsy shape
- Verify piece count text shows sub-piece info
- Verify URL param `wsp` persists across reload
- Download SVG → verify sub-puzzle paths present in the `<path>` element

**Commands:**
```bash
cargo test --manifest-path crates/puzzle-core/Cargo.toml -- sub_puzzle
cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- sub_piece
cargo check --target wasm32-unknown-unknown -p puzzle-wasm
wasm-pack build crates/puzzle-wasm --target web --out-dir pkg
```

## Constraints

- **K006 applies:** Whimsy config fields (including new `whimsy_sub_pieces`) must be extracted from config *before* `PuzzleGrid::new()` consumes the struct, since PuzzleGrid doesn't implement Clone.
- **RNG isolation (R013):** Sub-puzzle grid must use an isolated RNG seed (`"{seed}-whimsy-sub"`) so that adding/removing sub-puzzle config doesn't change the main grid's RNG sequence. The connector RNG also needs a distinct seed (`"{seed}-whimsy-sub-connectors"`), matching the existing `"{seed}-connectors"` pattern.
- **Fixed EDGE_STRIDE (36 floats):** Sub-puzzle connectors use the same ClassicKnobConnector with 5 cubic segments, so they produce the same 36-float stride. No binary format changes needed.
- **Minimum grid size:** PuzzleConfig validates `rows >= 2` and `cols >= 2`, so the minimum sub-puzzle is a 2×2 grid (up to 4 pieces inside the contour). This means the minimum useful sub-piece input is 2, which will create a 2×2 sub-grid.

## Common Pitfalls

- **Sub-grid coordinate space** — The sub-puzzle PuzzleGrid is sized to the whimsy *bounding box*, not the full puzzle. Its edges are in local whimsy coordinates (0,0 at top-left of bounding box). When exporting to binary/SVG, the edge coordinates must be offset by the whimsy bounding box origin. `BoundaryPuzzle` stores edges in grid coordinates, so the sub-grid's PuzzleConfig width/height should be the whimsy bounding box dimensions, and the exported edge positions need to be translated to puzzle-global mm coordinates.
- **Whimsy bounding box vs contour** — The sub-grid fills the *bounding box* of the whimsy shape, but cells are classified against the *contour*. For a star shape, many cells in the bounding box corners will be excluded. The user sees fewer sub-pieces than the grid's rows×cols. The UI should show actual sub-piece count, not target.
- **Empty sub-puzzle** — If `whimsy_sub_pieces` is set but `whimsy_shape` is not, or the whimsy scale is so small that no sub-grid cells fit inside, the sub-puzzle should be silently skipped (0 sub-pieces). Don't error.

## Open Risks

- **Very small whimsy shapes may produce 0 sub-pieces** — When the whimsy is scaled very small (0.2×), the sub-grid cells may all fall outside the contour. This is an edge case that should degrade gracefully (no sub-pieces shown, no error). The UI should reflect "0 of N target" or similar.
- **Sub-puzzle edge coordinate translation** — The sub-puzzle grid uses local coordinates (0,0 at whimsy bounding box origin). The binary export and SVG export assume coordinates are in puzzle-global mm space. The sub-grid must be created with dimensions matching the whimsy bounding box, and the exported data must be translated by the whimsy position offset. This is straightforward but must be done correctly — a single coordinate space mismatch would render sub-puzzle edges at the wrong position.
