---
phase: quick-16
plan: 16
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/puzzle-core/src/config.rs
  - crates/puzzle-wasm/src/lib.rs
  - web/src/main.ts
autonomous: true
requirements: [TAPER-RANGE-16]

must_haves:
  truths:
    - "User slider 0 maps to internal taper 0.57 (previously 0.50)"
    - "User slider 1 maps to internal taper 1.32 (previously 1.20)"
    - "User still sees 0.00-1.00 range on the slider"
    - "Existing URLs with old taper values are clamped to new range without errors"
    - "Randomize taper range also uses new 0.57-1.32 bounds"
  artifacts:
    - path: "crates/puzzle-core/src/config.rs"
      provides: "Updated validation bounds and default taper"
      contains: "0.57"
    - path: "crates/puzzle-wasm/src/lib.rs"
      provides: "Updated WASM clamp bounds"
      contains: "0.57"
    - path: "web/src/main.ts"
      provides: "Updated JS linear interpolation formula"
      contains: "0.75"
  key_links:
    - from: "web/src/main.ts"
      to: "crates/puzzle-core/src/config.rs"
      via: "buildConfig() maps user 0-1 to internal 0.57-1.32"
      pattern: "0\\.57.*0\\.75"
---

<objective>
Adjust the internal taper range from 0.50..=1.20 to 0.57..=1.32. The minimum moves from 0.50 to 0.57 (what user slider 0.1 previously mapped to). The maximum increases by 10% (1.20 * 1.10 = 1.32). The user-facing slider still shows 0-1, with the linear interpolation formula updated accordingly.

Purpose: Tighten the low end of the taper range (removing the mildest 10% which produces barely-visible taper) and extend the high end for more aggressive snap-fit options.
Output: Updated Rust config validation, WASM clamping, JS mapping formula, and passing tests.
</objective>

<execution_context>
@/home/caleb/.config/Claude/get-shit-done/workflows/execute-plan.md
@/home/caleb/.config/Claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@crates/puzzle-core/src/config.rs
@crates/puzzle-wasm/src/lib.rs
@web/src/main.ts

<interfaces>
From crates/puzzle-core/src/config.rs:
```rust
pub struct TabConfig {
    pub size_pct: f64,
    pub taper: f64,           // current valid range: 0.50..=1.20 → new: 0.57..=1.32
    pub size_pct_max: Option<f64>,
    pub taper_max: Option<f64>, // same range as taper
}

fn default_taper() -> f64 { 0.5 }  // → change to 0.57
impl TabConfig {
    pub fn validate(&self) -> Result<(), String> // bounds checks
    pub fn neck_ratio(&self) -> f64 { 1.0 - self.taper * 0.5 } // formula unchanged
    pub fn randomize_neck_ratio(&self, rng: &mut ChaCha8Rng) -> f64 // uses taper range
}
```

From web/src/main.ts (buildConfig):
```typescript
taper: 0.5 + parseFloat(taperSlider.value) * 0.7,         // → 0.57 + val * 0.75
taper_max: 0.5 + parseFloat(taperMaxSlider.value) * 0.7,  // → 0.57 + val * 0.75
```

From crates/puzzle-wasm/src/lib.rs (safe_tab_max):
```rust
config.tab.taper = config.tab.taper.clamp(0.50, 1.20);       // → clamp(0.57, 1.32)
*max = max.clamp(0.50, 1.20);                                  // → clamp(0.57, 1.32)
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Update Rust config validation, defaults, and WASM clamping</name>
  <files>
    crates/puzzle-core/src/config.rs
    crates/puzzle-wasm/src/lib.rs
  </files>
  <action>
Update all taper range bounds from 0.50..=1.20 to 0.57..=1.32 across both files.

In `crates/puzzle-core/src/config.rs`:
1. Change `default_taper()` return value from `0.5` to `0.57`
2. Change `TabConfig::default()` taper field from `0.5` to `0.57`
3. Update `validate()` taper bounds: `0.50` → `0.57`, `1.20` → `1.32` (both for `taper` and `taper_max`)
4. Update doc comments on the `taper` field: range description, mild/moderate/aggressive examples
5. Update `neck_ratio()` doc comment: taper=0.57 → ratio=0.715, taper=1.32 → ratio=0.34
6. Update ALL test cases that reference 0.50 or 1.20 as taper values:
   - `test_boundary_values_valid`: min taper 0.50→0.57, max taper 1.20→1.32, taper_max Some(1.20)→Some(1.32)
   - `test_randomize_neck_ratio_some_produces_range`: taper_max Some(1.20)→Some(1.32), and neck_ratio range comment/assertions: taper in [0.57, 1.32] → neck_ratio in [0.34, 0.715]
   - `test_validate_taper_max_out_of_range`: taper_max Some(1.50)→Some(1.50) is still out of range, no change needed — but update the base taper from 0.50→0.57
   - `test_validate_taper_max_less_than_min`: taper 0.80 is valid in new range, taper_max Some(0.50)→Some(0.57) (must still be less than 0.80 to trigger error — actually 0.50 is now below min so change to Some(0.60) which is < 0.80)
   - All other tests referencing taper: 0.50→0.57 for default values
   - `test_randomize_tab_size_some_produces_range`: taper field 0.50→0.57
   - `test_randomize_tab_size_none_returns_fixed`: uses TabConfig::default(), no change needed
   - `test_randomize_neck_ratio_none_returns_fixed`: uses TabConfig::default(), no change needed

In `crates/puzzle-wasm/src/lib.rs`:
1. Update `safe_tab_max()` clamp calls: `clamp(0.50, 1.20)` → `clamp(0.57, 1.32)` for both taper and taper_max lines (lines ~355 and ~360)
  </action>
  <verify>
    <automated>cargo test --manifest-path crates/puzzle-core/Cargo.toml -- --nocapture 2>&1 | tail -5</automated>
  </verify>
  <done>All Rust tests pass with new taper range 0.57..=1.32. Validation rejects values outside this range. Default taper is 0.57.</done>
</task>

<task type="auto">
  <name>Task 2: Update JS mapping formula and rebuild WASM</name>
  <files>
    web/src/main.ts
  </files>
  <action>
Update the JavaScript linear interpolation in `web/src/main.ts` to map user 0-1 to internal 0.57-1.32.

1. In `buildConfig()` function (~line 81): change `0.5 + parseFloat(taperSlider.value) * 0.7` → `0.57 + parseFloat(taperSlider.value) * 0.75`
2. In `buildConfig()` function (~line 87): change `0.5 + parseFloat(taperMaxSlider.value) * 0.7` → `0.57 + parseFloat(taperMaxSlider.value) * 0.75`
3. In `loadFromUrl()` URL parsing (~line 115-116): the existing code reads taper as 0-1 from URL and sets slider value — this is fine, no change needed (slider value is still 0-1, the mapping happens in buildConfig).

Then rebuild WASM:
```bash
cd crates/puzzle-wasm && wasm-pack build --target web --release --out-dir ../../web/pkg
```

Verify the built WASM works by running `npm run build` in the web directory to ensure no TS errors.
  </action>
  <verify>
    <automated>cd web && npm run build 2>&1 | tail -5</automated>
  </verify>
  <done>JS maps user slider 0→internal 0.57 and 1→internal 1.32. WASM rebuilt. Web project builds without errors. Old URL taper values (0-1 range) continue to work (clamped by WASM layer if they map outside new bounds).</done>
</task>

</tasks>

<verification>
1. `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — all config tests pass
2. `cd web && npm run build` — no TypeScript/build errors
3. Manual: open the app, set taper slider to 0 → internal value should be 0.57; set to 1 → should be 1.32
</verification>

<success_criteria>
- Internal taper range is 0.57..=1.32 (was 0.50..=1.20)
- User-facing slider still shows 0.00-1.00
- Linear mapping: `internal = 0.57 + user * 0.75`
- All Rust tests pass, web builds cleanly
- WASM binary rebuilt and deployed to web/pkg
</success_criteria>

<output>
After completion, create `.planning/quick/16-adjust-taper-parameter-range-min-stays-0/16-SUMMARY.md`
</output>
