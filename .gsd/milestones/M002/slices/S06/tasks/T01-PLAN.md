---
estimated_steps: 4
estimated_files: 1
---

# T01: Add R009/R010 SVG validation tests in puzzle-wasm

**Slice:** S06 — Export & Integration Polish
**Milestone:** M002

## Description

Add ~8 Rust tests to `crates/puzzle-wasm/src/lib.rs` that validate R009 (SVG export completeness) and R010 (geometric correctness). These tests prove the SVG export pipeline produces valid, complete, laser-cuttable output for all config combinations. All tests operate on SVG string content from `generate_svg()` — no `js_sys::Reflect::get` (panics on non-wasm targets per K009).

**Relevant skills:** `test` (for test writing patterns)

## Steps

1. **Add R009 tests — SVG completeness across 5 config combinations.** For each combination (rectangular, border-only/heart, whimsy-only/heart, border+whimsy, whimsy+sub-pieces), call `generate_svg()` and assert:
   - SVG starts with `<svg` and contains `</svg>`
   - Contains `viewBox` with correct dimensions
   - M-command count is within expected range (rect should have the most edges; border-only fewer; whimsy-only similar to border-only; combined the fewest)
   - For non-rectangular: path data contains `Z` commands (closed contours for boundary/hole)
   - For sub-pieces: more M commands than whimsy-without-sub-pieces (reusing the pattern from `test_generate_svg_with_sub_pieces_more_m_commands` but with additional assertions)

2. **Add R010 tests — geometric coordinate bounds.** Parse SVG path data from `generate_svg()` output for a whimsy+sub-pieces config:
   - Extract all numeric coordinate values from the SVG `d` attribute using simple string parsing (split on M/L/C/Z/space, parse floats)
   - Assert all X coordinates fall within `[-margin, width + margin]` where margin is `width * 0.25` (connector overshoot allowance)
   - Assert all Y coordinates fall within `[-margin, height + margin]` where margin is `height * 0.25`
   - Assert no degenerate subpaths: search for patterns like `M` followed immediately by another `M` or by `Z` with no intervening drawing commands (L/C/Q)

3. **Add R010 viewBox consistency test.** Extract `viewBox` attribute from SVG, parse width/height values, and assert they match the config's width/height values.

4. **Run the full test suite to confirm all existing tests still pass alongside new tests.**

## Must-Haves

- [ ] R009 test covering 5 config combinations (rect, border-only, whimsy-only, both, whimsy+sub-pieces) with SVG structure assertions
- [ ] R010 test asserting coordinate bounds with connector overshoot margin
- [ ] R010 test asserting no degenerate subpaths (M→M or M→Z without drawing commands)
- [ ] R010 test asserting viewBox matches config dimensions
- [ ] All 38 existing WASM tests continue to pass
- [ ] All 133 existing core tests continue to pass

## Verification

- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` — all tests pass (38 existing + new)
- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- r009` — new R009 tests pass
- `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml -- r010` — new R010 tests pass
- `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — 133 tests still pass

## Inputs

- `crates/puzzle-wasm/src/lib.rs` — existing test suite with `generate_svg()` function and 38 tests; test patterns for config JSON construction
- `crates/puzzle-core/src/svg_export.rs` — `build_svg_document()` format reference (single-quoted `d='...'` attribute, viewBox format)

## Expected Output

- `crates/puzzle-wasm/src/lib.rs` — ~8 new test functions added to the `#[cfg(test)] mod tests` block

## Key Context for Executor

## Observability Impact

- **New signals:** 8 new test functions filterable by `r009` and `r010` prefixes. Each produces detailed assertion messages with SVG content snippets, M-command counts, and coordinate values on failure.
- **Inspection:** `cargo test -- r009` runs R009 completeness tests; `cargo test -- r010` runs R010 geometric correctness tests. These are orthogonal filters that isolate different validation concerns.
- **Failure state:** If SVG generation regresses, tests identify which config combination fails (rect, border-only, whimsy-only, both, whimsy+sub-pieces) and what aspect broke (missing structure, out-of-bounds coordinates, degenerate paths, viewBox mismatch).

## Key Context for Executor

- **SVG format:** `<svg xmlns='...' width='{w}mm' height='{h}mm' viewBox='0 0 {w} {h}'><path d='...' stroke='#000000' stroke-width='0.001mm' fill='none'/></svg>` — single path element with all geometry in one `d` attribute. Single-quoted attributes (K013).
- **Config JSON patterns:** Look at existing tests like `test_generate_svg_with_whimsy()` and `test_whimsy_plus_border()` for the exact JSON format. Copy their config strings for the 5 combinations.
- **No js_sys in tests (K009):** All tests must call `generate_svg()` which returns a `String`, then assert on string content. Do NOT use `js_sys::Reflect::get` or any JsValue inspection.
- **Connector overshoot (research note):** Classic knob connectors extend beyond cell boundaries. Use 20-25% margin on coordinate bounds — e.g., for a 200×150 puzzle, valid X range is [-50, 250] and valid Y range is [-37.5, 187.5].
- **Degenerate path detection:** After splitting the `d` attribute content on whitespace and `M`/`Z` boundaries, check that between each `M` and the next `M` or `Z` there is at least one drawing command (a token starting with `L`, `C`, or `Q`, or numeric values for relative commands).
- **Naming convention:** Prefix test names with `test_r009_` and `test_r010_` so they can be filtered by requirement ID.
