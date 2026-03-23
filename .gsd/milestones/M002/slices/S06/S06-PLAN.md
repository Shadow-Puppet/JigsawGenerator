# S06: Export & Integration Polish

**Goal:** Validate that SVG export is complete and geometrically correct across all config combinations (R009, R010), fix download filename to include sub-piece count, and verify full URL round-trip with all params.
**Demo:** `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` passes with new R009/R010 validation tests; download filename includes sub-piece count when active; full URL round-trip preserves all state.

## Must-Haves

- R009 validated: tests prove SVG contains all geometry (border contour, whimsy cut line, sub-puzzle cuts, modified grid edges) for 5 config combinations
- R010 validated: tests prove geometric correctness — coordinates within puzzle bounds (with connector margin), no degenerate paths (empty M→M or M→Z), all boundary/hole contours closed with Z
- Download filename includes sub-piece count suffix when whimsy sub-pieces are active
- Full URL round-trip (including `wsp`) confirmed working
- All existing 38 WASM + 133 core tests continue to pass

## Proof Level

- This slice proves: final-assembly
- Real runtime required: yes (browser verification for filename and URL round-trip)
- Human/UAT required: no

## Verification

- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — all tests pass (existing 38 + ~8 new)
- `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — 133 tests still pass
- `grep -q 'sub-pieces' web/src/main.ts` — sub-piece suffix in download filename
- `npx tsc --noEmit` — TypeScript compiles clean (run from `web/` directory)
- R009 and R010 requirements updated to "validated" status

## Integration Closure

- Upstream surfaces consumed: `crates/puzzle-wasm/src/lib.rs` (generate_svg, generate_grid, generate_edges_binary endpoints), `web/src/main.ts` (download handler, updateURL/loadFromURL)
- New wiring introduced in this slice: none — validation tests and minor polish only
- What remains before the milestone is truly usable end-to-end: nothing — S06 is the final slice

## Observability / Diagnostics

- **Runtime signals:** `cargo test -- r009` and `cargo test -- r010` filter to R009/R010 tests specifically. Test failure output includes SVG snippets, M-command counts, and coordinate values for debugging.
- **Inspection surfaces:** Each test function name encodes which config combination is tested (rect, border-only, whimsy-only, both, whimsy+sub-pieces). Filtered test runs isolate R009 vs R010 concerns.
- **Failure visibility:** Assertion messages include actual vs expected values for M-command counts and coordinate ranges, making failures self-diagnosing.
- **Redaction constraints:** None — all test data is synthetic (no secrets, no PII).

## Tasks

- [ ] **T01: Add R009/R010 SVG validation tests in puzzle-wasm** `est:30m`
  - Why: R009 and R010 are "active" with "validate: unmapped" — these tests prove SVG completeness and geometric correctness, the core deliverable of S06
  - Files: `crates/puzzle-wasm/src/lib.rs`
  - Do: Add ~8 tests covering 5 config combinations for R009 (rect, border-only, whimsy-only, both, whimsy+sub-pieces) asserting SVG starts with `<svg`, has expected M-command count ranges, boundary/hole contours have Z closures; add R010 tests asserting coordinates within puzzle bounds (with ~20% margin for connector overshoot), no degenerate subpaths (M immediately followed by M or Z), sub-puzzle coordinates within whimsy bounding box. All tests operate on SVG string content — no js_sys::Reflect::get (K009).
  - Verify: `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — all tests pass including new ones
  - Done when: All new R009/R010 tests pass, existing 38 tests still pass, `cargo test --manifest-path crates/puzzle-core/Cargo.toml` still passes 133 tests

- [ ] **T02: Add sub-piece count to download filename and verify full URL round-trip** `est:20m`
  - Why: Download filename is missing sub-piece count suffix (currently `puzzle-6x8-heart-whimsy-seed-abc.svg` even with 4 sub-pieces). URL round-trip with all params needs verification. Piece count display verified working in S05 browser tests.
  - Files: `web/src/main.ts`
  - Do: In download handler (~line 1573), add sub-piece suffix to filename when `whimsy_sub_pieces >= 2` (e.g., `-4sp`). Build WASM, start dev server, browser-verify: (1) download filename with heart border + star whimsy + 4 sub-pieces includes all suffixes, (2) full URL round-trip — set all params → copy URL → reload → verify identical state including `wsp`.
  - Verify: `grep -q 'sub.*pieces\|SubPieces\|sp' web/src/main.ts` confirms sub-piece suffix in filename code; `npx tsc --noEmit` from `web/` compiles clean
  - Done when: Download filename includes sub-piece count, TypeScript compiles clean, browser confirms URL round-trip with all params

## Files Likely Touched

- `crates/puzzle-wasm/src/lib.rs`
- `web/src/main.ts`
