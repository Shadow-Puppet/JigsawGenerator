---
phase: quick
plan: 002
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/puzzle-core/src/config.rs
  - web/index.html
  - web/src/main.ts
autonomous: true
requirements: [quick-002]

must_haves:
  truths:
    - "Slider shows 0.00-1.00 range to user"
    - "Internal taper value sent to Rust is 0.5 when slider at 0, 1.2 when slider at 1"
    - "URL taper param stores 0-100 (integer percentage of user-facing 0-1 range)"
    - "Old URLs with taper values outside 0-100 are clamped to valid range"
    - "neck_ratio formula still produces correct narrowing across new internal range"
  artifacts:
    - path: "crates/puzzle-core/src/config.rs"
      provides: "Validation for internal taper range 0.50..=1.20"
    - path: "web/index.html"
      provides: "Slider with min=0 max=1 step=0.01"
    - path: "web/src/main.ts"
      provides: "Mapping function: user 0-1 → internal 0.5-1.2"
  key_links:
    - from: "web/src/main.ts (buildConfig)"
      to: "Rust TabConfig.taper"
      via: "linear interpolation: internal = 0.5 + slider_value * 0.7"
      pattern: "0\\.5 \\+ .* \\* 0\\.7"
---

<objective>
Change taper to use a normalized 0-1 user-facing slider that maps to an internal 0.5-1.2 range.

Purpose: Simplify the user experience (0=no taper, 1=max taper) while expanding the effective internal range for connector generation.
Output: Updated Rust validation, HTML slider, and TypeScript mapping.
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
Update TabConfig in config.rs:

1. Change taper field comment to: `Taper amount controlling the neck-to-body ratio (0.50..=1.20, default 0.50). 0.50 = mild taper, 0.85 = moderate snap-fit, 1.20 = aggressive taper (narrow neck, wide body). Note: the UI presents this as a normalized 0-1 range; the WASM layer maps user 0→internal 0.5, user 1→internal 1.2.`

2. Change `default_taper()` return value to `0.5` (this stays the same numerically but semantically it's now the bottom of the range).

3. Change validation in `TabConfig::validate()`:
   - From: `self.taper < 0.30 || self.taper > 1.10`
   - To: `self.taper < 0.50 || self.taper > 1.20`
   - Update error message to: `"tab taper must be between 0.50 and 1.20, got {}"`

4. Update `neck_ratio()` comment to: `taper=0.50 → ratio=0.75 (mild), taper=0.85 → ratio=0.575, taper=1.20 → ratio=0.40 (aggressive)`. The formula `1.0 - self.taper * 0.5` stays unchanged — it naturally works with the new range (0.5→0.75, 1.2→0.40).

5. Update boundary tests:
   - `test_boundary_values_valid`: Change both `taper: 0.30` to `taper: 0.50` (min boundary) and `taper: 1.10` to `taper: 1.20` (max boundary, appears twice).
  </action>
  <verify>
    <automated>cargo test --manifest-path crates/puzzle-core/Cargo.toml -- --nocapture 2>&1 | tail -20</automated>
  </verify>
  <done>Rust validation accepts 0.50..=1.20 range, boundary tests pass with new limits, neck_ratio produces 0.75 at taper=0.50 and 0.40 at taper=1.20</done>
</task>

<task type="auto">
  <name>Task 2: Update HTML slider and TypeScript mapping</name>
  <files>web/index.html, web/src/main.ts</files>
  <action>
**index.html changes:**
1. Change taper slider attributes from `min="0.30" max="1.10" step="0.01" value="0.5"` to `min="0" max="1" step="0.01" value="0"` (user-facing 0 = mildest taper, which maps to internal 0.5).
2. Change default readout from `0.50` to `0.00` in the span `id="taper-readout"`.

**main.ts changes:**

1. In `buildConfig()`, change the taper value from raw slider to mapped:
   - From: `taper: parseFloat(taperSlider.value)`
   - To: `taper: 0.5 + parseFloat(taperSlider.value) * 0.7`
   This maps user 0→0.5, user 1→1.2 (linear interpolation).

2. In `loadFromURL()`:
   - Change taper parsing: the URL param `taper` stores integer percentage of the user-facing 0-1 range (so `taper=50` means user value 0.5).
   - From:
     ```
     const taperRaw = parseInt(params.get("taper") ?? "50", 10) / 100;
     const taper = Math.max(0.30, Math.min(1.10, taperRaw));
     ```
   - To:
     ```
     const taperUser = parseInt(params.get("taper") ?? "0", 10) / 100;
     const taper = Math.max(0, Math.min(1, taperUser));
     ```
   This clamps to 0-1 user-facing range. The internal mapping happens in `buildConfig()`.

3. In `updateURL()`:
   - Change taper URL encoding to store the user-facing slider value (0-1 as integer percentage):
   - From: `params.set("taper", String(Math.round(tabObj.taper * 100)));`
   - To: `params.set("taper", String(Math.round(parseFloat(taperSlider.value) * 100)));`
   This stores the raw slider value (0-100 integer), not the internal mapped value.

4. In `updateReadouts()`:
   - The taper readout line already does `parseFloat(taperSlider.value).toFixed(2)` which will now show 0.00-1.00. No change needed.
  </action>
  <verify>
    <automated>npm run build 2>&1 | tail -10</automated>
  </verify>
  <done>HTML slider shows 0-1 range, TypeScript maps user 0→internal 0.5 and user 1→internal 1.2, URL stores user-facing 0-100 integer, old URLs with out-of-range values are clamped to 0-1</done>
</task>

</tasks>

<verification>
1. `cargo test --manifest-path crates/puzzle-core/Cargo.toml` — all Rust tests pass
2. `npm run build` — WASM + Vite build succeeds
3. Manual check: slider at 0 should produce mild taper (internal 0.5), slider at 1 should produce aggressive taper (internal 1.2)
</verification>

<success_criteria>
- Rust validates taper in 0.50..=1.20 range
- HTML slider shows min=0 max=1
- buildConfig() sends 0.5 + slider * 0.7 to WASM
- URL param stores slider value as 0-100 integer
- All tests pass, build succeeds
</success_criteria>

<output>
After completion, create `.planning/quick/002-change-taper-range-to-0-5-1-2-internally/002-SUMMARY.md`
</output>
