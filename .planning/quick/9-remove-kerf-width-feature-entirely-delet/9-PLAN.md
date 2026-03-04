---
phase: 9-remove-kerf
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/puzzle-core/src/kerf.rs (DELETE)
  - crates/puzzle-core/src/lib.rs
  - crates/puzzle-core/src/config.rs
  - crates/puzzle-core/src/svg_export.rs
  - crates/puzzle-core/src/grid.rs
  - crates/puzzle-wasm/src/lib.rs
  - web/src/main.ts
  - web/index.html
autonomous: true
requirements: [KERF-REMOVE]

must_haves:
  truths:
    - "No kerf_width field exists anywhere in the Rust structs"
    - "No kerf slider or readout appears in the UI"
    - "No kerf URL parameter is read or written"
    - "cargo test passes for puzzle-core and puzzle-wasm"
    - "WASM builds successfully"
    - "Web app loads and generates puzzles without errors"
  artifacts:
    - path: "crates/puzzle-core/src/kerf.rs"
      provides: "DELETED — must not exist"
    - path: "crates/puzzle-core/src/config.rs"
      provides: "PuzzleConfig without kerf_width field"
    - path: "crates/puzzle-core/src/svg_export.rs"
      provides: "generate_svg without kerf offset logic"
    - path: "web/src/main.ts"
      provides: "UI code without kerf references"
    - path: "web/index.html"
      provides: "HTML without kerf slider"
  key_links:
    - from: "crates/puzzle-core/src/svg_export.rs"
      to: "crates/puzzle-core/src/kerf.rs"
      via: "use crate::kerf::offset_path — MUST BE REMOVED"
      pattern: "kerf"
---

<objective>
Remove the kerf compensation feature entirely from the codebase. The kerf offset algorithm never worked correctly and the user wants it gone — delete the module, config field, UI controls, and URL parameter.

Purpose: Clean up broken feature; simplify codebase
Output: Working puzzle generator with no kerf references; WASM rebuilt
</objective>

<execution_context>
@.planning/STATE.md
</execution_context>

<context>
This is a complete removal across three layers: Rust core, WASM bridge, and web frontend.

Key files and what to remove from each:
- `kerf.rs` — entire file deleted
- `lib.rs` (puzzle-core) — `pub mod kerf;` and `pub use kerf::*;` lines
- `config.rs` — `kerf_width` field from PuzzleConfig struct, Default impl, validate() check, `from_input()` parameter, and 3 kerf-specific tests
- `svg_export.rs` — `use crate::kerf::offset_path;` import, the kerf offset block in `generate_svg()`, the `kerf_width: 0.0` in test helper, and `test_kerf_only_offsets_border` test
- `grid.rs` — `kerf_width: 0.0` in test helper
- `lib.rs` (puzzle-wasm) — kerf doc comments on `generate_svg`, `test_generate_svg_with_kerf` test, `test_generate_svg_backward_compat` test (tests kerf default)
- `main.ts` — `kerfSlider`, `kerfReadout` variables, `kerf_width` in buildConfig, `kerf` in URL read/write, kerf in sliders array, kerf readout update
- `index.html` — kerf slider HTML block
</context>

<tasks>

<task type="auto">
  <name>Task 1: Remove kerf from Rust core (puzzle-core)</name>
  <files>
    crates/puzzle-core/src/kerf.rs
    crates/puzzle-core/src/lib.rs
    crates/puzzle-core/src/config.rs
    crates/puzzle-core/src/svg_export.rs
    crates/puzzle-core/src/grid.rs
  </files>
  <action>
1. **Delete** `crates/puzzle-core/src/kerf.rs` entirely.

2. **Edit `crates/puzzle-core/src/lib.rs`:**
   - Remove line `pub mod kerf;`
   - Remove line `pub use kerf::*;`

3. **Edit `crates/puzzle-core/src/config.rs`:**
   - Remove the `kerf_width` field and its doc comment from the `PuzzleConfig` struct (lines 217-221: the doc comment about kerf compensation, the `#[serde(default)]` attr, and `pub kerf_width: f64`)
   - Remove `kerf_width: 0.0,` from `Default for PuzzleConfig` impl (line 235)
   - Remove the kerf validation check from `validate()` (lines 257-262: the `if self.kerf_width < 0.0 || self.kerf_width > 1.0` block)
   - Remove the `kerf_width` parameter from `from_input()` — remove it from the function signature (line 281) and from the struct literal (line 292)
   - Remove `0.0,` kerf_width argument from all `from_input()` calls in tests (there are 5 test calls: `test_from_input_inches_converts`, `test_from_input_mm_no_conversion`, `test_from_input_validates`, and 3 in `test_boundary_values_valid`)
   - Remove 3 kerf-specific tests entirely: `test_validate_kerf_negative`, `test_validate_kerf_too_large`, `test_default_kerf_zero`

4. **Edit `crates/puzzle-core/src/svg_export.rs`:**
   - Remove import line: `use crate::kerf::offset_path;`
   - Remove kerf offset block from `generate_svg()` (lines 25-27): the `if grid.config.kerf_width > 0.0 { border = offset_path(...); }` block
   - Also remove the doc comment line about kerf: `/// - Optional kerf compensation when \`config.kerf_width > 0\`` (line 20)
   - Remove `kerf_width: 0.0,` from the `test_config` helper in the tests module (line 250)
   - Remove the entire `test_kerf_only_offsets_border` test (lines 411-459)

5. **Edit `crates/puzzle-core/src/grid.rs`:**
   - Remove `kerf_width: 0.0,` from the `test_config` helper in the tests module (line 259)

After all edits, run: `cargo test -p puzzle-core`
  </action>
  <verify>
    <automated>cargo test -p puzzle-core 2>&1</automated>
  </verify>
  <done>All kerf references removed from puzzle-core; `cargo test -p puzzle-core` passes with 0 failures; no "kerf" string remains in any puzzle-core source file.</done>
</task>

<task type="auto">
  <name>Task 2: Remove kerf from WASM bridge and web frontend, rebuild</name>
  <files>
    crates/puzzle-wasm/src/lib.rs
    web/src/main.ts
    web/index.html
  </files>
  <action>
1. **Edit `crates/puzzle-wasm/src/lib.rs`:**
   - Remove the doc comment lines mentioning kerf from `generate_svg` function (lines 208, 214): `/// \`kerf_width\` field (defaults to 0.0 if omitted for backward compatibility).` and `/// - Optional kerf compensation when \`kerf_width > 0\``
   - Remove the entire `test_generate_svg_with_kerf` test (lines 446-464)
   - Remove the entire `test_generate_svg_backward_compat` test (lines 484-494) — this was specifically testing kerf_width defaulting

2. **Edit `web/src/main.ts`:**
   - Remove the `kerfSlider` variable declaration (line 34): `let kerfSlider: HTMLInputElement;`
   - Remove the `kerfReadout` variable declaration (line 43): `let kerfReadout: HTMLElement;`
   - Remove `kerf_width: parseFloat(kerfSlider.value),` from `buildConfig()` (line 93)
   - In `loadFromURL()`: remove `const kerf = parseFloat(params.get("kerf") ?? "0");` (line 113) and `kerfSlider.value = String(kerf);` (line 124)
   - In `updateURL()`: remove `params.set("kerf", String(config.kerf_width));` (line 157)
   - Remove `kerfSlider` from the DOM cache section: `kerfSlider = document.getElementById("kerf") as HTMLInputElement;` (line 410)
   - Remove `kerfReadout` from DOM cache: `kerfReadout = document.getElementById("kerf-readout")!;` (line 419)
   - Remove `kerfSlider` from the `sliders` array (line 460): change `[tabSlider, taperSlider, radiusSlider, kerfSlider]` → `[tabSlider, taperSlider, radiusSlider]`
   - Remove the kerf readout update from `updateReadouts()` (line 304): `kerfReadout.textContent = parseFloat(kerfSlider.value).toFixed(2);`

3. **Edit `web/index.html`:**
   - Remove the entire kerf slider block (lines 87-93):
     ```html
     <div class="slider-group">
       <div class="slider-label">
         <label for="kerf">Kerf Width</label>
         <span class="readout" id="kerf-readout">0.00</span>
       </div>
       <input type="range" id="kerf" min="0" max="1" step="0.01" value="0" />
     </div>
     ```

4. **Run WASM tests:** `cargo test -p puzzle-wasm`

5. **Rebuild WASM:** `wasm-pack build crates/puzzle-wasm --target web --release --out-dir ../../web/pkg`

6. **Verify no "kerf" references remain:** `grep -ri kerf crates/ web/src/ web/index.html` should return nothing.
  </action>
  <verify>
    <automated>cargo test -p puzzle-wasm 2>&1 && wasm-pack build crates/puzzle-wasm --target web --release --out-dir ../../web/pkg 2>&1 && echo "--- Checking for leftover kerf references ---" && grep -ri kerf crates/puzzle-core/src/ crates/puzzle-wasm/src/ web/src/ web/index.html || echo "CLEAN: No kerf references found"</automated>
  </verify>
  <done>WASM tests pass, WASM rebuilt successfully to web/pkg/, zero "kerf" references in any source file across Rust and web layers. The kerf feature is completely removed.</done>
</task>

</tasks>

<verification>
1. `cargo test -p puzzle-core` — all tests pass, no kerf tests remain
2. `cargo test -p puzzle-wasm` — all tests pass, no kerf tests remain
3. `grep -ri kerf crates/puzzle-core/src/ crates/puzzle-wasm/src/ web/src/ web/index.html` — returns nothing
4. `ls crates/puzzle-core/src/kerf.rs` — file does not exist
5. WASM package in web/pkg/ is freshly built
</verification>

<success_criteria>
- kerf.rs deleted
- No `kerf` string in any Rust source or web source
- `cargo test` passes for both puzzle-core and puzzle-wasm
- WASM rebuilt to web/pkg/
- Web app generates puzzles (manual check: `npx vite` and load localhost)
</success_criteria>

<output>
After completion, create `.planning/quick/9-remove-kerf-width-feature-entirely-delet/9-SUMMARY.md`
</output>
