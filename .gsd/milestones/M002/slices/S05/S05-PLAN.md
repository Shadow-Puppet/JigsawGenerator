# S05: Whimsy Sub-Puzzle Splitting

**Goal:** A placed whimsy shape splits into N sub-pieces with working connectors inside the whimsy contour, visible in Canvas preview and included in SVG export.
**Demo:** User places a heart whimsy, sets sub-pieces to 4, and sees internal connector cut lines inside the heart shape. Downloaded SVG contains these sub-puzzle paths.

## Must-Haves

- `whimsy_sub_pieces: Option<u32>` field in PuzzleConfig with `#[serde(default)]`
- Sub-grid dimension computation: rows×cols from target N and whimsy aspect ratio
- Sub-puzzle generation reusing `BoundaryPuzzle::new(sub_grid, whimsy_contour)` with isolated RNG seed
- Sub-puzzle edge/border data appended to main puzzle binary output (same EDGE_STRIDE format)
- Sub-puzzle connector paths included in SVG export
- Coordinate translation from sub-grid local space to puzzle-global mm space
- UI numeric input (2–16 range) for sub-piece count, visible only when whimsy is active
- URL param `wsp` for sub-piece count persistence
- Graceful degradation when whimsy is too small for sub-pieces (0 sub-pieces, no error)

## Proof Level

- This slice proves: integration (Rust core → WASM binary → Canvas rendering pipeline)
- Real runtime required: yes (browser verification for Canvas rendering)
- Human/UAT required: no (automated Rust tests + browser assertions)

## Verification

- `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- sub_puzzle` — unit tests for sub-grid computation, BoundaryPuzzle-inside-whimsy generation, edge count consistency, determinism
- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- sub_piece` — WASM tests for SVG output with sub-pieces, backward compatibility, piece count
- `cargo check --target wasm32-unknown-unknown -p puzzle-wasm` — WASM compilation
- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- sub_pieces_no_whimsy_skips` — graceful degradation: sub-pieces silently skipped when whimsy is not active (no error returned)
- Browser: place heart whimsy → set sub-pieces to 4 → verify internal cut lines visible → verify URL param `wsp` persists → download SVG → verify sub-puzzle paths present

## Observability / Diagnostics

- Runtime signals: WASM error JSON for invalid sub-piece configs; `piece_count` in response unchanged (main puzzle only); sub-puzzle edge count visible via binary data length increase
- Inspection surfaces: URL param `wsp` reflects sub-piece count; piece count text includes sub-piece info suffix; SVG path `d` attribute M-command count increases with sub-pieces
- Failure visibility: coordinate translation errors visible as sub-puzzle edges drawn at wrong position (top-left corner instead of whimsy location); zero sub-pieces when whimsy too small degrades silently
- Redaction constraints: none

## Integration Closure

- Upstream surfaces consumed: `resolve_whimsy_shape()` from `crates/puzzle-wasm/src/lib.rs` (whimsy contour), `BoundaryPuzzle` from `crates/puzzle-core/src/boundary.rs` (grid clipping), `PuzzleGrid::new()` + `generate_connectors()` from `crates/puzzle-core/src/grid.rs`, `EDGE_STRIDE` binary format from `crates/puzzle-core/src/binary_export.rs`, whimsy state variables from `web/src/main.ts`
- New wiring introduced in this slice: sub-puzzle generation helper in WASM lib, `whimsy_sub_pieces` config field flowing through all three endpoints, sub-piece count UI input wired to `buildConfig()`
- What remains before the milestone is truly usable end-to-end: S06 — SVG export validation, URL param round-trip for full state, R010 geometry correctness, R013 determinism verification

## Tasks

- [ ] **T01: Implement sub-puzzle generation in Rust core and wire into WASM endpoints** `est:1h`
  - Why: The core risk — proves grid-inside-whimsy approach works with correct coordinate translation, RNG isolation, and binary/SVG export. Without this, T02 has nothing to render.
  - Files: `crates/puzzle-core/src/config.rs`, `crates/puzzle-wasm/src/lib.rs`
  - Do: (1) Add `whimsy_sub_pieces: Option<u32>` to PuzzleConfig with serde default. (2) In WASM lib, add a `generate_sub_puzzle()` helper that: computes sub-grid rows/cols from target N and whimsy bounding box aspect ratio, creates a PuzzleConfig with whimsy bounding box dimensions and `"{seed}-whimsy-sub"` seed, generates a PuzzleGrid + connectors, wraps in `BoundaryPuzzle::new(sub_grid, whimsy_contour)`, translates edge coordinates by whimsy bounding box origin, returns binary edge data + border data + SVG path data. (3) Wire into all three WASM endpoints — when `whimsy_sub_pieces` is `Some(n)` and whimsy is active, call `generate_sub_puzzle()` and append its data to the main puzzle output. (4) Add unit tests for sub-grid dimension computation, sub-puzzle cell count > 0, edge count consistency, and determinism. (5) Add WASM tests for SVG with sub-pieces (more M commands), backward compat, piece count. Constraints: extract `whimsy_sub_pieces` from config before `PuzzleGrid::new()` consumes it (K006). Minimum sub-grid is 2×2. Skip sub-puzzle silently when whimsy is not active or sub-pieces is None.
  - Verify: `cargo test --manifest-path crates/puzzle-core/Cargo.toml -- sub_puzzle && cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- sub_piece && cargo check --target wasm32-unknown-unknown -p puzzle-wasm`
  - Done when: All sub_puzzle and sub_piece tests pass, WASM compiles, and `generate_svg` with `whimsy_sub_pieces` produces SVG with more M commands than without sub-pieces.

- [ ] **T02: Add sub-piece count UI input with Canvas rendering and URL persistence** `est:30m`
  - Why: Makes sub-puzzle visible to users and closes the slice demo — internal cut lines appear in the Canvas preview when sub-pieces are set.
  - Files: `web/index.html`, `web/src/main.ts`
  - Do: (1) Add numeric input (id="whimsy-sub-pieces", type="number", min=2, max=16, step=1) to the Whimsy Shape section in index.html, hidden by default, shown when whimsy is active. (2) In main.ts, add `whimsySubPieces` state variable (number, 0 = disabled). Wire the input's change event to set state and call `scheduleGenerate()`. (3) In `buildConfig()`, include `whimsy_sub_pieces: whimsySubPieces` when whimsySubPieces > 0 and whimsy is active. (4) Add `wsp` URL param read/write — read in `restoreFromUrl()`, write in `updateUrl()`. (5) Update piece count text to include sub-piece info (e.g. "44 pieces (heart whimsy, 4 sub-pieces)"). (6) Clear sub-piece state in `clearWhimsy()`. (7) Show/hide sub-pieces input when whimsy dropdown changes. (8) Rebuild WASM with `wasm-pack build crates/puzzle-wasm --target web --out-dir pkg` (K010). No new Canvas drawing code needed — sub-puzzle edges arrive in the existing Float64Array format and are drawn by `drawVisibleEdges()` and `drawBorder()`.
  - Verify: `wasm-pack build crates/puzzle-wasm --target web --out-dir pkg` succeeds. Browser: place heart whimsy → set sub-pieces to 4 → internal cut lines visible → URL shows `wsp=4` → reload preserves state.
  - Done when: Sub-piece input appears when whimsy is active, internal cut lines render in Canvas, SVG download includes sub-puzzle paths, and URL param `wsp` persists.

## Files Likely Touched

- `crates/puzzle-core/src/config.rs` — add `whimsy_sub_pieces` field
- `crates/puzzle-wasm/src/lib.rs` — sub-puzzle generation helper, endpoint wiring, WASM tests
- `web/index.html` — sub-piece count numeric input
- `web/src/main.ts` — sub-piece state, buildConfig, URL params, piece count display
