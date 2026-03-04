---
phase: quick-001
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/puzzle-core/src/config.rs
  - web/index.html
  - web/src/main.ts
autonomous: true
requirements: [QUICK-001]

must_haves:
  truths:
    - "Taper slider minimum is 0.30 (users cannot set taper below 0.30)"
    - "Taper slider maximum is 1.10 (allowing ~10% more taper than before)"
    - "Default taper value of 0.50 still works and is within new range"
    - "Existing URL-shared puzzles with taper values in old range still load correctly"
    - "WASM build succeeds and puzzle generates correctly at new boundary values"
  artifacts:
    - path: "crates/puzzle-core/src/config.rs"
      provides: "Updated taper validation bounds (0.30..=1.10) and neck_ratio formula"
    - path: "web/index.html"
      provides: "Taper slider with min=0.30, max=1.10"
    - path: "web/src/main.ts"
      provides: "URL param decoding with clamping to new taper range"
  key_links:
    - from: "web/index.html"
      to: "web/src/main.ts"
      via: "slider min/max must match validation in buildConfig"
    - from: "web/src/main.ts"
      to: "crates/puzzle-core/src/config.rs"
      via: "taper value from JS must pass Rust validation"
---

<objective>
Adjust the taper range so that the minimum is 0.30 (previously 0.0) and the maximum is 1.10 (previously 1.0), giving ~10% more taper range for narrower necks.

Purpose: The current minimum taper of 0.0 produces no neck narrowing at all (cylindrical tabs), which is not useful for laser-cut puzzles. Setting 0.30 as the floor ensures all generated puzzles have visible snap-fit necks. Extending the max to 1.10 allows even more aggressive taper for tighter fits.

Output: Updated Rust validation, HTML slider bounds, and JS URL handling.
</objective>

<execution_context>
@/home/caleb/.config/Claude/get-shit-done/workflows/execute-plan.md
@/home/caleb/.config/Claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@crates/puzzle-core/src/config.rs
@web/index.html
@web/src/main.ts
</context>

<tasks>

<task type="auto">
  <name>Task 1: Update Rust taper validation and neck_ratio formula</name>
  <files>crates/puzzle-core/src/config.rs</files>
  <action>
In `crates/puzzle-core/src/config.rs`:

1. Change `TabConfig::validate()` taper bounds from `0.0..=1.0` to `0.30..=1.10`:
   - Line 67: `if self.taper < 0.30 || self.taper > 1.10 {`
   - Line 69: Update error message: `"tab taper must be between 0.30 and 1.10, got {}"`

2. The `neck_ratio()` formula `1.0 - self.taper * 0.5` already works correctly with the new range:
   - At taper=0.30: neck_ratio = 0.85 (mild narrowing)
   - At taper=1.10: neck_ratio = 0.45 (aggressive narrowing)
   - No formula change needed.

3. Update the `default_taper()` function: keep it at 0.5 (still within range, no change needed).

4. Update tests that use boundary values:
   - In `test_boundary_values_valid` (line ~385): change `taper: 0.5` in the minimum config to `taper: 0.30` to test the new minimum.
   - In `test_boundary_values_valid` (line ~402): change `taper: 1.0` in the maximum config to `taper: 1.10` to test the new maximum (appears twice, update both).

5. Update doc comment on `taper` field (line 38): change `(0.0..=1.0, default 0.5)` to `(0.30..=1.10, default 0.5)`.
  </action>
  <verify>
    Run `cargo test -p puzzle-core` — all tests pass including boundary tests with new values.
    Run `cargo build -p puzzle-core` — compiles without warnings.
  </verify>
  <done>Taper validation accepts 0.30..=1.10, rejects values outside. Default 0.5 is valid. All existing tests pass with updated boundaries.</done>
</task>

<task type="auto">
  <name>Task 2: Update HTML slider and JS URL handling for new taper range</name>
  <files>web/index.html, web/src/main.ts</files>
  <action>
In `web/index.html`:

1. Update taper slider (line 71):
   - Change `min="0"` to `min="0.30"`
   - Change `max="1"` to `max="1.10"`
   - Keep `step="0.01"` and `value="0.5"`

2. Update the default readout (line 69): keep `0.50` (still valid default).

In `web/src/main.ts`:

1. In `loadFromURL()` (line 75): Update the default fallback and add clamping.
   Change the taper parsing line:
   ```typescript
   const taper = parseInt(params.get("taper") ?? "50", 10) / 100; // 50 → 0.5
   ```
   To:
   ```typescript
   const taperRaw = parseInt(params.get("taper") ?? "50", 10) / 100;
   const taper = Math.max(0.30, Math.min(1.10, taperRaw)); // clamp to valid range
   ```
   This ensures old shared URLs with taper=0 get clamped to 0.30 instead of failing Rust validation.

2. No changes needed to `updateURL()` — it already correctly encodes the current slider value as an integer percentage. The new range (30-110) maps fine.

3. No changes needed to `updateReadouts()` — it already shows `toFixed(2)` for taper.
  </action>
  <verify>
    Run `npm run build` from `web/` directory — builds without errors.
    Inspect built HTML to confirm slider has min="0.30" max="1.10".
  </verify>
  <done>Taper slider range is 0.30-1.10 in the UI. Old URL params with out-of-range taper values are clamped to valid range. WASM build succeeds.</done>
</task>

<task type="auto">
  <name>Task 3: Rebuild WASM and verify end-to-end</name>
  <files></files>
  <action>
1. Build the WASM module: `wasm-pack build crates/puzzle-wasm --target web --release`
2. Run the full test suite: `cargo test --workspace`
3. Verify the web build: `npm run build` from `web/` directory

This ensures the Rust validation change propagates through WASM to the web frontend without errors.
  </action>
  <verify>
    `wasm-pack build crates/puzzle-wasm --target web --release` succeeds.
    `cargo test --workspace` — all tests pass.
    `npm run build` in web/ — builds successfully.
  </verify>
  <done>Full pipeline builds and all tests pass with new taper range 0.30..=1.10.</done>
</task>

</tasks>

<verification>
1. `cargo test --workspace` — all Rust tests pass
2. `npm run build` in web/ — frontend builds
3. Taper slider in HTML has min="0.30" max="1.10"
4. Rust validation rejects taper=0.0 and taper=1.5
5. Rust validation accepts taper=0.30 and taper=1.10
</verification>

<success_criteria>
- Taper range is 0.30..=1.10 in Rust validation, HTML slider, and JS URL clamping
- All existing tests pass with updated boundary values
- Full WASM + web build succeeds
- Default taper of 0.50 still works correctly
</success_criteria>

<output>
After completion, create `.planning/quick/001-adjust-taper-range-make-30-the-minimum-a/001-SUMMARY.md`
</output>
