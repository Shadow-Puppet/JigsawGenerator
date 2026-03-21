---
phase: quick-011
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/puzzle-core/src/edge.rs
  - crates/puzzle-core/src/grid.rs
  - crates/puzzle-core/src/classic_connector.rs
autonomous: true
must_haves:
  truths:
    - "Knobs on horizontal and vertical edges have the same visual size for any grid aspect ratio"
    - "Opposing knobs never overlap, even at extreme aspect ratios (e.g., 5:1)"
    - "Existing tests pass — seed determinism, connector continuity, and edge invariants preserved"
  artifacts:
    - path: "crates/puzzle-core/src/edge.rs"
      provides: "EdgeParams with cross_length field"
      contains: "cross_length"
    - path: "crates/puzzle-core/src/grid.rs"
      provides: "Cross-axis-aware connector generation and safe_tab_max"
      contains: "cross_length"
    - path: "crates/puzzle-core/src/classic_connector.rs"
      provides: "Knob scaling using min(length, cross_length)"
      contains: "cross_length"
  key_links:
    - from: "grid.rs generate_connectors()"
      to: "EdgeParams"
      via: "passes cross_length (cell_w for v-edges, cell_h for h-edges)"
      pattern: "cross_length"
    - from: "classic_connector.rs generate()"
      to: "EdgeParams.cross_length"
      via: "uses min(length, cross_length) for knob base dimension"
      pattern: "min.*cross_length"
---

<objective>
Fix vertical axis knob scaling to match horizontal axis scaling and prevent knob overlapping.

Purpose: Currently, knob size scales as a fraction of the edge length (`knob_w = length * tab_size`). Since h-edges have length=cell_w and v-edges have length=cell_h, non-square grids produce differently-sized knobs on each axis. Additionally, `safe_tab_max()` floors at 0.15, which at extreme aspect ratios (e.g., 5:1 piece ratio) causes opposing knobs to overlap.

Output: Uniform knob sizing across both axes using min(cell_w, cell_h) as the knob base dimension, and a fixed safe_tab_max that properly prevents overlap at all aspect ratios.
</objective>

<execution_context>
@/home/caleb/.config/Claude/get-shit-done/workflows/execute-plan.md
@/home/caleb/.config/Claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@crates/puzzle-core/src/edge.rs
@crates/puzzle-core/src/grid.rs
@crates/puzzle-core/src/classic_connector.rs
@crates/puzzle-core/src/connector.rs
@crates/puzzle-core/src/config.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add cross_length to EdgeParams and update grid connector generation</name>
  <files>
    crates/puzzle-core/src/edge.rs
    crates/puzzle-core/src/grid.rs
  </files>
  <action>
1. In `edge.rs`, add a `cross_length: f64` field to `EdgeParams`:
   ```rust
   pub struct EdgeParams {
       pub length: f64,
       pub cross_length: f64, // perpendicular cell dimension (cell_h for h-edges, cell_w for v-edges)
       pub direction: TabDirection,
       pub tab_size: f64,
       pub neck_ratio: f64,
   }
   ```

2. In `grid.rs` `generate_connectors()`:
   - Compute `cell_w = self.config.width / cols as f64` and `cell_h = self.config.height / rows as f64` at the top of the method.
   - For h-edge EdgeParams: set `cross_length: cell_h`
   - For v-edge EdgeParams: set `cross_length: cell_w`

3. In `grid.rs` `safe_tab_max()`: Fix the floor logic. Instead of `.max(0.15)`, use a conditional: if `theoretical_max * 0.9 < 0.15`, still return `theoretical_max * 0.9` (no floor). This prevents forcing a tab size that causes overlap. The UI-side clamping in TabConfig will handle the display range. Update the formula comments to reflect this change.
   Change: `(theoretical_max * 0.9).max(0.15).min(0.25)` → `(theoretical_max * 0.9).min(0.25)`

4. Update ALL `EdgeParams` construction sites in tests (both in `edge.rs` tests and `connector.rs` tests) to include `cross_length: 50.0` (or appropriate test values).

5. Update the `NullConnector` test in `connector.rs` to include `cross_length` in EdgeParams.
  </action>
  <verify>
    <automated>cargo test --manifest-path crates/puzzle-core/Cargo.toml 2>&1 | tail -20</automated>
  </verify>
  <done>EdgeParams has cross_length field, all construction sites updated, safe_tab_max no longer forces minimum 0.15 for extreme ratios, all existing tests compile and pass</done>
</task>

<task type="auto">
  <name>Task 2: Use cross_length for uniform knob scaling in ClassicKnobConnector</name>
  <files>
    crates/puzzle-core/src/classic_connector.rs
  </files>
  <action>
1. In `ClassicKnobConnector::generate()`, change the knob base dimension from `length` to `length.min(params.cross_length)`:
   ```rust
   let base = length.min(params.cross_length);
   let knob_w = base * params.tab_size;
   let knob_h = knob_w * KNOB_HEIGHT_RATIO * dir_sign;
   ```
   This ensures both axes produce identically-sized knobs based on the smaller cell dimension.

   Keep `center = length * 0.5` (knob is still centered on the actual edge).
   Keep the first bezier segment starting at `(0.0, 0.0)` and the last ending at `(length, 0.0)`.
   
   The approach control point `center - knob_w * APPROACH_RATIO` may need adjustment. Since `knob_w` is now potentially smaller than before (when cross_length < length), the approach points will be closer to center, which is fine — the flat baseline sections on either side of the knob will just be longer.

2. In `validate()`, update the bounding box check: the Y-axis bounding check should use `cross_length` (not `length`) since knobs protrude into the perpendicular cell dimension:
   ```rust
   let cross = params.cross_length;
   // ...
   if bbox.y0 < -cross - margin || bbox.y1 > cross + margin {
   ```
   Actually, the Y-axis check should bound knob height against `cross_length / 2` (half the perpendicular dimension, since opposing knobs share the space). But the existing margin logic uses `length * 0.05`. Update the margin to use `base.min(cross)` for a reasonable bound, or simply keep `length * 0.05` as-is since the key fix is in the scaling itself.

   Simplest approach: keep validate() mostly as-is but use `params.cross_length` for the Y bound:
   ```rust
   let margin = params.length * 0.05;
   if bbox.x0 < -margin
       || bbox.x1 > params.length + margin
       || bbox.y0 < -params.cross_length - margin
       || bbox.y1 > params.cross_length + margin
   ```

3. Update tests in `classic_connector.rs`:
   - All `default_params()` and `EdgeParams` construction: add `cross_length: 50.0` (matching the existing `length: 50.0` for square test case).
   - Add a new test `test_uniform_knob_size_across_axes` that creates two EdgeParams with different lengths but same cross_length (simulating h-edge and v-edge of a non-square grid), generates connectors, and asserts the max Y extent (knob height) is identical for both.
   - Add a new test `test_extreme_aspect_ratio_no_overlap` that uses `length: 100.0, cross_length: 20.0` and verifies the knob's max Y extent is less than `cross_length / 2` (10mm), confirming opposing knobs wouldn't overlap.

4. Rebuild WASM after all Rust changes:
   ```bash
   wasm-pack build crates/puzzle-wasm --target web --release
   ```
  </action>
  <verify>
    <automated>cargo test --manifest-path crates/puzzle-core/Cargo.toml 2>&1 | tail -30 && wasm-pack build crates/puzzle-wasm --target web --release 2>&1 | tail -5</automated>
  </verify>
  <done>Knobs use min(length, cross_length) for sizing, producing uniform knob dimensions across axes. New tests confirm: (1) same knob height regardless of axis for non-square grids, (2) no overlap at extreme aspect ratios. WASM rebuilt successfully.</done>
</task>

</tasks>

<verification>
1. `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — all tests pass
2. `wasm-pack build crates/puzzle-wasm --target web --release` — WASM builds
3. Visual check: generate puzzle with non-square dimensions (e.g., 297x105mm, 8x3 grid) — knobs should be same size on both axes and not overlap
</verification>

<success_criteria>
- All existing tests pass (seed determinism, connector continuity, edge coordinates)
- New tests verify uniform knob sizing and no-overlap guarantee
- WASM builds successfully
- Knobs visually identical in size on both horizontal and vertical edges
- No overlap at any valid grid configuration including extreme aspect ratios
</success_criteria>

<output>
After completion, create `.planning/quick/11-fix-vertical-axis-knob-scaling-to-match-/11-SUMMARY.md`
</output>
