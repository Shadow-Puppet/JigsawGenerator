# S06: Export & Integration Polish — Research

**Date:** 2026-03-21
**Depth:** Light research — straightforward validation and polish on well-established patterns

## Summary

S06 is a validation and polish slice. The core export pipeline is already complete — `generate_edges_binary()` and `generate_svg()` both handle all four border×whimsy combinations, include sub-puzzle data, and cache the SVG for download. The download handler already produces correct filenames with border and whimsy suffixes. URL params already persist all state (rows, cols, dims, seed, border, ws/wx/wy/wsc/wsp).

The remaining work is **validating** R009 and R010 (which are currently "active" with "validate: unmapped") through new tests, plus minor polish:
1. **R009 validation** — prove that downloaded SVG contains all geometry (border contour, whimsy cut line, sub-puzzle cuts, modified grid edges) for all config combinations.
2. **R010 validation** — prove geometric correctness: no duplicate paths, coordinates within puzzle bounds, path well-formedness (every subpath starts with M, closed paths have Z).
3. **Download filename polish** — add sub-piece count to filename when active (currently missing).
4. **Full URL round-trip test** — verify that all params (including wsp) survive a full round-trip through `updateURL()` → `loadFromURL()` → `buildConfig()`.
5. **Piece count display correctness** — verify piece_count text correctly reflects all combinations.

All five items are testable through existing Rust unit tests (R009/R010) and structural grep checks (URL/filename/display). No new architecture or risky integration.

## Recommendation

Split into two tasks:
- **T01 (Rust tests):** Add validation tests in `puzzle-wasm` for R009 (SVG completeness across combinations) and R010 (geometric correctness — coordinates in bounds, no degenerate paths, proper M/Z structure). Update requirements to "validated".
- **T02 (JS polish + browser verification):** Add sub-piece count to download filename, verify full URL round-trip with all params, verify piece count display for all combinations via browser. Final integrated acceptance: heart border, star whimsy with sub-pieces, download SVG, reload from URL.

## Implementation Landscape

### Key Files

- `crates/puzzle-wasm/src/lib.rs` — All WASM endpoints and the test suite (38 tests). T01 adds ~6-8 new tests here for R009/R010.
- `web/src/main.ts` — Download handler (line ~1577), `updateURL()` / `loadFromURL()`, piece count display in `generatePuzzle()`. T02 makes minor edits.
- `crates/puzzle-core/src/boundary.rs` — BoundaryPuzzle with SVG/binary export. Already fully tested (20 tests). No changes expected.
- `crates/puzzle-core/src/svg_export.rs` — `build_svg_document()` format. Read-only reference for test assertions.

### Build Order

**T01 first** — Rust tests proving SVG/geometric correctness are the core deliverable. Tests cover:
- R009: SVG completeness for 5 combinations (rect, border-only, whimsy-only, both, whimsy+sub-pieces). Each test asserts SVG starts with `<svg`, contains expected M-command count ranges, includes Z closures for boundary/hole contours, has correct viewBox dimensions.
- R010: Geometric correctness. Parse SVG path data, verify all coordinate values fall within `[0, width]` × `[0, height]` (with small margin for connector overshoot). Verify no empty subpaths (M immediately followed by M or Z). Verify sub-puzzle coordinates fall within whimsy bounding box.

**T02 second** — JS polish. Add sub-piece count to download filename. Browser verification of full URL round-trip and piece count display.

### Verification Approach

1. `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — all tests pass (existing 38 + new ~6-8)
2. `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — 133 tests still pass (no changes)
3. `npx tsc --noEmit` — TypeScript compiles clean
4. `wasm-pack build crates/puzzle-wasm --target web --out-dir pkg` — WASM builds
5. Browser: Download SVG with heart border + star whimsy + 4 sub-pieces → open SVG in browser/editor → visually confirm all cut paths present
6. Browser: Full URL round-trip — set all params → copy URL → paste in new tab → verify identical state
7. R009/R010 requirements updated to "validated" with test names

## Constraints

- Tests for R010 must work at the SVG string level (parsing path data) since we don't have an SVG parser crate. Simple regex/string matching for coordinate extraction is sufficient.
- `js_sys::Reflect::get` panics on non-wasm targets (K009) — all tests must verify via Rust domain logic or SVG string content, not JsValue inspection.
- `append_sub_puzzle_to_svg()` assumes single-quoted `d` attribute (K013) — tests should verify this assumption holds.

## Common Pitfalls

- **Coordinate margin for connector overshoot** — Classic knob connectors extend slightly beyond the edge bounding box (tab protrusions). R010 coordinate bounds check must allow ~15-20% margin beyond puzzle dimensions for connector curves that legitimately extend past cell boundaries.
- **Sub-puzzle coordinate range** — Sub-puzzle paths are translated to puzzle-global coords and should fall within the whimsy bounding box, not within the full puzzle bounds. The test must compute the whimsy bbox from the config to set appropriate bounds.
