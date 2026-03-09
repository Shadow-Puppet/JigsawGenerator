---
phase: "17"
plan: 1
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/puzzle-core/src/config.rs
  - crates/puzzle-core/src/svg_export.rs
  - crates/puzzle-core/src/binary_export.rs
  - crates/puzzle-core/src/grid.rs
  - crates/puzzle-wasm/src/lib.rs
  - web/src/main.ts
  - web/index.html
autonomous: true
requirements: [QUICK-17]
must_haves:
  truths:
    - "Corner radius slider no longer appears in the UI"
    - "Border is rendered with sharp 90-degree corners (simple rectangle)"
    - "URL no longer includes a radius parameter"
    - "All existing Rust tests pass with border field removed"
    - "WASM accepts config JSON without border field"
  artifacts:
    - path: "crates/puzzle-core/src/config.rs"
      provides: "PuzzleConfig without BorderConfig struct or border field"
    - path: "crates/puzzle-core/src/svg_export.rs"
      provides: "Simple rectangular border path (no arcs, no append_quarter_arc)"
    - path: "web/index.html"
      provides: "No corner radius slider group"
    - path: "web/src/main.ts"
      provides: "No radius slider references, no radius URL param"
  key_links:
    - from: "web/src/main.ts"
      to: "puzzle-wasm"
      via: "buildConfig() no longer sends border field"
      pattern: "buildConfig.*border"
---

<objective>
Remove the corner radius property entirely from the puzzle generator codebase.

Purpose: Simplify the codebase by eliminating the unused/unwanted rounded corner feature. Sharp corners are the desired default for laser-cut puzzles.
Output: Clean codebase with no BorderConfig, no radius UI, simple rectangular border path.
</objective>

<execution_context>
@/home/caleb/.config/Claude/get-shit-done/workflows/execute-plan.md
@/home/caleb/.config/Claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@crates/puzzle-core/src/config.rs
@crates/puzzle-core/src/svg_export.rs
@crates/puzzle-core/src/binary_export.rs
@crates/puzzle-core/src/grid.rs
@crates/puzzle-wasm/src/lib.rs
@web/src/main.ts
@web/index.html
</context>

<tasks>

<task type="auto">
  <name>Task 1: Remove BorderConfig from Rust core and simplify border path to rectangle</name>
  <files>
    crates/puzzle-core/src/config.rs
    crates/puzzle-core/src/svg_export.rs
    crates/puzzle-core/src/binary_export.rs
    crates/puzzle-core/src/grid.rs
    crates/puzzle-wasm/src/lib.rs
  </files>
  <action>
**config.rs:**
- Delete the entire `BorderConfig` struct (lines 168-192: the struct, Default impl, and validate impl)
- Remove `pub border: BorderConfig` field from `PuzzleConfig` (line 214)
- Remove `border: BorderConfig::default()` from `PuzzleConfig::default()` (line 228)
- Remove `self.border.validate()?;` from `PuzzleConfig::validate()` (line 252)
- Remove `border: BorderConfig` parameter from `PuzzleConfig::from_input()` (line 267) and its usage in the struct literal (line 277)
- Delete tests `test_validate_border_negative` and `test_validate_border_too_large` (lines 376-388)
- In ALL remaining test calls to `PuzzleConfig::from_input(...)`, remove the `BorderConfig::default()` argument (lines 399, 417, 435) and the `BorderConfig { corner_radius: ... }` arguments (lines 456, 474, 494)
- In ALL `PuzzleConfig { ... }` struct literals in tests, remove the `border: BorderConfig::default()` line
- Add `#[serde(default)]` skip: since existing URLs/configs might still send `"border": {...}`, add a serde `deny_unknown_fields` would break. Instead, just remove the field. Serde will ignore unknown fields by default (serde's default behavior is to silently ignore unknown JSON fields when deserializing, which handles backward compat).

**svg_export.rs:**
- Simplify `build_border_path()` (lines 68-117) to a simple rectangle: `move_to(0,0)`, `line_to(w,0)`, `line_to(w,h)`, `line_to(0,h)`, `close_path()`. No radius, no arcs.
- Delete the `append_quarter_arc()` function entirely (lines 190-207)
- Remove `use std::f64::consts::PI;` (line 1) — no longer needed (check if PI is used elsewhere in the file first; it's only used for arcs)
- Remove `Arc, Vec2` from the kurbo import (line 3) — only needed for arcs. Keep `Affine, BezPath, PathEl, Point`.
- In test `test_config()`, remove `border: BorderConfig::default()` (line 242)

**binary_export.rs:**
- In test `test_config()`, remove `border: BorderConfig::default()` (line 170)
- Update import if `BorderConfig` is imported from config (check — it's used via `crate::config::*` wildcard so just removing usage is enough)

**grid.rs:**
- In test `test_config()`, remove `border: BorderConfig::default()` (line 269)

**puzzle-wasm/src/lib.rs:**
- Update the doc comment JSON example (line 98) to remove the `"border"` field
- In ALL test config JSON strings, remove `,"border":{"corner_radius":2.0}` from every test string (lines 378, 407, 416, 433, 440, 463, 487, 513, 526, 540)
  </action>
  <verify>
    Run `cargo test --workspace` from project root — all tests must pass.
    Run `cargo clippy --workspace` — no warnings.
  </verify>
  <done>
    BorderConfig struct deleted. PuzzleConfig has no border field. build_border_path generates simple rectangle. append_quarter_arc deleted. All Rust tests pass. WASM accepts JSON without border field.
  </done>
</task>

<task type="auto">
  <name>Task 2: Remove radius slider from UI and URL params, rebuild WASM</name>
  <files>
    web/index.html
    web/src/main.ts
  </files>
  <action>
**web/index.html:**
- Delete the entire corner radius slider group (lines 73-78): the `<div class="slider-group">` containing the "Corner Radius" label, `radius-readout` span, and `#radius` range input.

**web/src/main.ts:**
- Remove `let radiusSlider: HTMLInputElement;` declaration (line 21)
- Remove `let radiusReadout: HTMLElement;` declaration (line 28)
- In `buildConfig()`: remove `border: { corner_radius: parseFloat(radiusSlider.value) }` from the returned object (line 96)
- In `loadFromURL()`: remove `const radius = parseFloat(params.get("radius") ?? "2");` (line 114) and `radiusSlider.value = String(radius);` (line 126)
- In `updateURL()`: remove `const borderObj = config.border as { corner_radius: number };` (line 149) and `params.set("radius", String(borderObj.corner_radius));` (line 158)
- In `updateReadouts()`: remove `radiusReadout.textContent = parseFloat(radiusSlider.value).toFixed(1);` (line 479)
- In DOM cache section: remove `radiusSlider = document.getElementById("radius") as HTMLInputElement;` (line 756) and `radiusReadout = document.getElementById("radius-readout")!;` (line 763)
- In slider event listeners: change `const sliders = [tabSlider, taperSlider, radiusSlider];` to `const sliders = [tabSlider, taperSlider];` (line 850)

**Build WASM:**
- Run `wasm-pack build crates/puzzle-wasm --target web --release` (or the project's established build command)
- Copy the built WASM + JS glue files to `web/pkg/` if needed

**Verify UI:**
- Run `npm run dev` (or equivalent) briefly to confirm no console errors
  </action>
  <verify>
    Run `wasm-pack build crates/puzzle-wasm --target web --release` succeeds.
    Run `npx tsc --noEmit` in web/ — no TypeScript errors.
    Grep for "radius" in web/src/main.ts — should find zero matches (or only unrelated occurrences).
    Grep for "BorderConfig" across entire codebase — should find zero matches.
  </verify>
  <done>
    Corner radius slider removed from HTML. All radius references removed from TypeScript. WASM rebuilt without border field. No TypeScript errors. No "BorderConfig" or "corner_radius" references remain in the codebase.
  </done>
</task>

</tasks>

<verification>
- `cargo test --workspace` — all Rust tests pass
- `cargo clippy --workspace` — no warnings
- `wasm-pack build crates/puzzle-wasm --target web --release` — builds clean
- `grep -r "BorderConfig\|corner_radius\|radiusSlider\|radiusReadout\|radius-readout\|append_quarter_arc" crates/ web/src/` — zero matches
- `grep -r '"radius"' web/src/main.ts` — zero matches (no URL param references)
</verification>

<success_criteria>
- BorderConfig struct and all references completely removed from Rust codebase
- Border path is a simple rectangle (4 line segments, no arcs)
- Corner radius slider removed from HTML
- All radius-related JS/TS code removed
- URL no longer includes radius parameter
- Old URLs with radius param still work (serde ignores unknown fields; JS loadFromURL simply won't read the param)
- WASM rebuilt and all tests pass
</success_criteria>

<output>
After completion, create `.planning/quick/17-remove-corner-radius-property-entirely-f/17-SUMMARY.md`
</output>
