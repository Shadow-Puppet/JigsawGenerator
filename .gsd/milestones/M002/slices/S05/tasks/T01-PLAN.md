---
estimated_steps: 8
estimated_files: 2
---

# T01: Implement sub-puzzle generation in Rust core and wire into WASM endpoints

**Slice:** S05 — Whimsy Sub-Puzzle Splitting
**Milestone:** M002

## Description

Add `whimsy_sub_pieces` config field to PuzzleConfig and implement sub-puzzle generation that creates a mini-puzzle inside the whimsy contour. The sub-puzzle reuses the existing `BoundaryPuzzle::new()` + `PuzzleGrid` pipeline with the whimsy shape as the boundary. Sub-puzzle edge and border data are appended to the main puzzle's binary output and SVG path data. All coordinate translation from sub-grid local space to puzzle-global mm space is handled in the WASM layer.

**Relevant skills:** None specific — this is Rust core + WASM work using established patterns.

## Steps

1. **Add `whimsy_sub_pieces` to PuzzleConfig** in `crates/puzzle-core/src/config.rs`:
   - Add `#[serde(default, skip_serializing_if = "Option::is_none")] pub whimsy_sub_pieces: Option<u32>` field
   - Add it to `Default::default()` as `None`
   - Add it to `from_input()` as `None`
   - Update all test helpers that construct PuzzleConfig manually (search for `test_config` in `config.rs`, `boundary.rs`, `grid.rs`, `svg_export.rs`, `binary_export.rs`) — add `whimsy_sub_pieces: None`

2. **Implement sub-grid dimension computation** as a helper function `compute_sub_grid_dims(target_n: u32, bbox_w: f64, bbox_h: f64) -> (u32, u32)` in `crates/puzzle-wasm/src/lib.rs`:
   - Compute aspect ratio `r = bbox_w / bbox_h`
   - `rows = max(2, round(sqrt(target_n / r)))`
   - `cols = max(2, round(rows * r))`
   - If `rows * cols` differs from target by more than 2, adjust the smaller dimension by ±1
   - Return `(rows, cols)`, both clamped to `[2, 100]`
   - Add a unit test `test_sub_grid_dimensions` verifying several target values

3. **Implement `generate_sub_puzzle()` helper** in `crates/puzzle-wasm/src/lib.rs`:
   - Signature: `fn generate_sub_puzzle(whimsy_contour: &kurbo::BezPath, target_pieces: u32, seed: &str) -> Option<SubPuzzleData>` where `SubPuzzleData` is a new struct with `edges: Vec<f64>`, `border: Vec<f64>`, `svg_path: String`, `cell_count: usize`
   - Compute the whimsy contour's bounding box using `kurbo::Shape::bounding_box()`
   - Call `compute_sub_grid_dims(target_pieces, bbox_w, bbox_h)`
   - Create a `PuzzleConfig` with: rows/cols from above, width=bbox_w, height=bbox_h, seed=`"{seed}-whimsy-sub"`, default tab config
   - Create `PuzzleGrid::new(sub_config)` and `generate_connectors(&ClassicKnobConnector)`
   - Create the boundary for the sub-puzzle: translate the whimsy contour so it's in local coordinates (subtract bbox origin from all points)
   - Wrap in `BoundaryPuzzle::new(sub_grid, local_whimsy_contour)`
   - If `included_cell_count() == 0`, return `None` (graceful degradation)
   - Get binary edge data via `boundary_edges_to_binary()` and translate all coordinates by bbox origin `(bbox_x, bbox_y)` — every edge's start/end/control points shift by this offset
   - Get binary border data via `boundary_border_to_binary()` and translate all coordinates similarly (MoveTo and LineTo: +2 floats after cmd, CurveTo: +6 floats after cmd)
   - Build SVG path fragment from the translated sub-puzzle's `generate_boundary_svg()` (extract path `d` attribute content) or build it directly from BoundaryPuzzle — actually simpler to just generate the SVG path data by iterating the BoundaryPuzzle's included edges and boundary directly, applying the translation offset. See existing `generate_boundary_svg()` for pattern.
   - Return `Some(SubPuzzleData { edges, border, svg_path, cell_count })`

4. **Wire `generate_sub_puzzle()` into `generate_edges_binary()`** endpoint:
   - After extracting whimsy fields (already done per K006), also extract `whimsy_sub_pieces`
   - In each of the four match arms that has a whimsy path (whimsy-only and both), after computing the main puzzle data:
     - If `whimsy_sub_pieces` is `Some(n)` and `n >= 2`, call `generate_sub_puzzle(&whimsy_path, n, &seed)` where `seed` is the original seed string
     - If it returns `Some(sub)`, extend `edges_data` with `sub.edges` and `border_data` with `sub.border`
   - For match arms without whimsy (neither, border-only), skip sub-puzzle entirely

5. **Wire into `generate_svg()`** endpoint:
   - Same extraction and branching as step 4
   - When sub-puzzle is generated, the SVG path needs to include sub-puzzle paths
   - The simplest approach: after constructing the main `BoundaryPuzzle` and calling `generate_boundary_svg()`, we can't easily append to the SVG string. Instead, modify the match arms to build a combined BezPath that includes both the main puzzle and sub-puzzle geometry, then pass to `build_svg_document()`. Or: generate the sub-puzzle SVG path data separately and insert it into the main SVG's `<path d='...'>` attribute before the closing `'`. Either way, the sub-puzzle's connector paths and boundary need to appear in the final SVG `d` attribute.
   - Recommended approach: generate the main SVG string, then if sub-puzzle data exists, extract the `d='...'` content, append the sub-puzzle's SVG path fragment, and reconstruct. This avoids modifying the BoundaryPuzzle API.

6. **Wire into `generate_grid()`** endpoint:
   - Extract `whimsy_sub_pieces` before PuzzleGrid::new()
   - When whimsy and sub-pieces are active, compute the sub-puzzle's cell count
   - Add a `sub_piece_count` field to the grid response (or just return it alongside main piece count)
   - This is informational — the main `piece_breakdown.total` stays as the main puzzle piece count

7. **Write Rust unit tests** at bottom of `crates/puzzle-wasm/src/lib.rs` (in `mod tests`):
   - `test_sub_grid_dimensions`: verify computation for several inputs (e.g., N=4 with square bbox → 2×2, N=9 with 2:1 bbox → 3×3 or 2×4)
   - `test_generate_svg_with_sub_pieces`: SVG with `whimsy_sub_pieces: 4` has more M commands than same config without sub-pieces
   - `test_sub_pieces_backward_compat`: JSON without `whimsy_sub_pieces` field produces same output as before
   - `test_sub_pieces_deterministic`: same seed + same config = identical SVG

8. **Verify WASM compilation**: `cargo check --target wasm32-unknown-unknown -p puzzle-wasm`

## Must-Haves

- [ ] `whimsy_sub_pieces: Option<u32>` in PuzzleConfig with serde default
- [ ] Sub-grid dimension computation produces rows×cols ≈ target N with aspect ratio matching
- [ ] Sub-puzzle uses isolated RNG seed (`"{seed}-whimsy-sub"`) — R013 determinism
- [ ] Sub-puzzle edge coordinates translated from local to puzzle-global mm space
- [ ] Sub-puzzle edges use same EDGE_STRIDE (36 floats) binary format
- [ ] Sub-puzzle connector paths appear in SVG output
- [ ] Graceful skip when whimsy not active or sub-pieces is None/0
- [ ] All existing tests still pass (backward compatibility)

## Verification

- `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — all existing + new tests pass
- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — all existing + new sub_piece tests pass
- `cargo check --target wasm32-unknown-unknown -p puzzle-wasm` — WASM target compiles
- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- sub_piece` — sub-piece-specific tests pass

## Observability Impact

- **New signal:** `sub_piece_count` field in `generate_grid()` JSON response — present only when whimsy and sub-pieces are active, absent otherwise (inspectable via JSON parse of grid response)
- **Binary data length increase:** `edges` Float64Array grows by `sub_edge_count * EDGE_STRIDE` floats when sub-pieces are generated — observable by comparing edge array lengths with vs without `whimsy_sub_pieces`
- **SVG M-command count:** SVG path `d` attribute contains more M commands when sub-pieces are active — verifiable by counting `M` occurrences in the SVG string
- **Failure visibility:** `generate_sub_puzzle()` returns `None` (graceful skip) when whimsy bounding box < 1mm or zero cells inside the contour — no error surfaced, but sub-puzzle data is absent from output
- **Inspection command:** `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- sub_piece` runs all sub-piece tests

## Inputs

- `crates/puzzle-core/src/config.rs` — PuzzleConfig struct to extend
- `crates/puzzle-core/src/boundary.rs` — BoundaryPuzzle API (new, new_with_hole, generate_boundary_svg, boundary_edges_to_binary, boundary_border_to_binary)
- `crates/puzzle-core/src/grid.rs` — PuzzleGrid::new, generate_connectors
- `crates/puzzle-core/src/binary_export.rs` — EDGE_STRIDE, CMD_* constants
- `crates/puzzle-core/src/seed.rs` — create_rng for understanding RNG isolation pattern
- `crates/puzzle-wasm/src/lib.rs` — existing WASM endpoints, resolve_whimsy_shape(), four-combination match logic

## Expected Output

- `crates/puzzle-core/src/config.rs` — PuzzleConfig with `whimsy_sub_pieces: Option<u32>` field
- `crates/puzzle-wasm/src/lib.rs` — `compute_sub_grid_dims()`, `generate_sub_puzzle()`, `SubPuzzleData` struct, updated all three endpoints, 4+ new tests
- `crates/puzzle-core/src/grid.rs` — test_config helper updated with whimsy_sub_pieces field
- `crates/puzzle-core/src/boundary.rs` — test_config helper updated with whimsy_sub_pieces field
- `crates/puzzle-core/src/svg_export.rs` — test_config helper updated (if present)
- `crates/puzzle-core/src/binary_export.rs` — test_config helper updated (if present)
