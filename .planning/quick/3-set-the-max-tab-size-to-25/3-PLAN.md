---
phase: quick
plan: 3
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/puzzle-core/src/config.rs
  - crates/puzzle-core/src/grid.rs
  - crates/puzzle-core/src/classic_connector.rs
  - web/index.html
autonomous: true
requirements: []
must_haves:
  truths:
    - "Tab size slider cannot exceed 25%"
    - "Rust validation rejects tab size_pct > 0.25"
    - "All existing tests pass with updated bounds"
  artifacts:
    - path: "crates/puzzle-core/src/config.rs"
      provides: "TabConfig validation with 0.25 upper bound"
      contains: "0.25"
    - path: "crates/puzzle-core/src/grid.rs"
      provides: "safe_tab_max clamped to 0.25"
      contains: ".min(0.25)"
    - path: "web/index.html"
      provides: "Slider max attribute set to 0.25"
      contains: 'max="0.25"'
  key_links:
    - from: "crates/puzzle-core/src/config.rs"
      to: "crates/puzzle-core/src/grid.rs"
      via: "validate() and safe_tab_max() share same upper bound"
      pattern: "0\\.25"
---

<objective>
Change the maximum tab size from 45% to 25% across Rust validation, grid clamping, and web UI.

Purpose: Cap the tab size at 25% to prevent overly large tabs that produce ugly or overlapping connectors.
Output: Updated validation bounds, grid clamping, HTML slider max, and passing tests.
</objective>

<execution_context>
@/home/caleb/.config/Claude/get-shit-done/workflows/execute-plan.md
@/home/caleb/.config/Claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md

Key files:
@crates/puzzle-core/src/config.rs — TabConfig validation (line 62: 0.15..=0.45 → 0.15..=0.25)
@crates/puzzle-core/src/grid.rs — safe_tab_max() clamp (line 167: .min(0.45) → .min(0.25))
@crates/puzzle-core/src/classic_connector.rs — test uses tab_size: 0.45 in proportion test (line 366)
@web/index.html — slider max attribute (line 57: max="0.45" → max="0.25")
</context>

<tasks>

<task type="auto">
  <name>Task 1: Update Rust validation and grid clamping</name>
  <files>
    crates/puzzle-core/src/config.rs
    crates/puzzle-core/src/grid.rs
    crates/puzzle-core/src/classic_connector.rs
  </files>
  <action>
In `config.rs`:
- Line 34 doc comment: change `0.15..=0.45` to `0.15..=0.25`
- Line 62: change `self.size_pct > 0.45` to `self.size_pct > 0.25`
- Line 64: change error message from "0.15 and 0.45" to "0.15 and 0.25"
- Line 296 test `test_validate_tab_too_large`: change `config.tab.size_pct = 0.50` to `config.tab.size_pct = 0.30` (still above 0.25, still triggers error)
- Line 384 and 401/420 in `test_boundary_values_valid`: change `size_pct: 0.45` to `size_pct: 0.25` (maximum valid boundary)

In `grid.rs`:
- Line 167: change `.min(0.45)` to `.min(0.25)` in safe_tab_max()

In `classic_connector.rs`:
- Line 366 in `test_tab_size_affects_proportions`: change `tab_size: 0.45` to `tab_size: 0.25` for the large_params (still larger than the small 0.15, still tests proportionality)
  </action>
  <verify>
    <automated>cargo test --manifest-path crates/puzzle-core/Cargo.toml 2>&1 | tail -5</automated>
  </verify>
  <done>Rust validation rejects size_pct > 0.25, safe_tab_max clamps to 0.25, all tests pass</done>
</task>

<task type="auto">
  <name>Task 2: Update HTML slider max</name>
  <files>web/index.html</files>
  <action>
In `web/index.html` line 57:
- Change `max="0.45"` to `max="0.25"` on the tab slider input element.

No TypeScript changes needed — `main.ts` already dynamically updates `tabSlider.max` via `updateTabMax()` which calls `safe_tab_max()` from WASM. The HTML attribute is just the initial fallback default, and the WASM function will now return values clamped to 0.25 via `grid.rs`.
  </action>
  <verify>
    <automated>grep 'id="tab"' web/index.html | grep 'max="0.25"'</automated>
  </verify>
  <done>HTML slider max attribute is 0.25, matching the new Rust upper bound</done>
</task>

</tasks>

<verification>
```bash
# Full Rust test suite passes
cargo test --manifest-path crates/puzzle-core/Cargo.toml

# WASM crate tests pass (uses tab configs internally)
cargo test --manifest-path crates/puzzle-wasm/Cargo.toml

# HTML slider has correct max
grep 'id="tab"' web/index.html | grep 'max="0.25"'

# Config validation error message updated
grep '0.15 and 0.25' crates/puzzle-core/src/config.rs

# Grid clamping updated
grep '.min(0.25)' crates/puzzle-core/src/grid.rs
```
</verification>

<success_criteria>
- Tab size validation rejects values > 0.25 (was 0.45)
- safe_tab_max() clamps to 0.25 ceiling (was 0.45)
- HTML slider default max is 0.25
- All Rust tests pass (puzzle-core and puzzle-wasm crates)
</success_criteria>

<output>
After completion, create `.planning/quick/3-set-the-max-tab-size-to-25/3-SUMMARY.md`
</output>
