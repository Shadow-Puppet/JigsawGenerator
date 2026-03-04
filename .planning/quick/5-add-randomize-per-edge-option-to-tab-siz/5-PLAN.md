---
phase: quick-005
plan: 5
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/puzzle-core/src/config.rs
  - crates/puzzle-core/src/grid.rs
  - crates/puzzle-wasm/src/lib.rs
  - web/index.html
  - web/src/main.ts
  - web/src/style.css
autonomous: true
requirements: [QUICK-005]

must_haves:
  truths:
    - "User can toggle randomize-per-edge mode for tab size via checkbox"
    - "User can toggle randomize-per-edge mode for taper via checkbox"
    - "When checkbox enabled, slider becomes dual-thumb range selector showing min/max"
    - "Each internal edge gets a random tab_size/taper within the selected range"
    - "Puzzle remains deterministic (same seed produces same per-edge values)"
    - "URL sharing preserves randomize mode and range values"
  artifacts:
    - path: "crates/puzzle-core/src/config.rs"
      provides: "TabConfig with optional range fields"
      contains: "size_pct_max"
    - path: "crates/puzzle-core/src/grid.rs"
      provides: "Per-edge random tab_size and neck_ratio in generate_connectors"
      contains: "size_pct_max"
    - path: "web/index.html"
      provides: "Dual-range slider HTML with checkbox toggles"
      contains: "tab-randomize"
    - path: "web/src/main.ts"
      provides: "Range slider logic, config builder, URL sync"
      contains: "tab-randomize"
  key_links:
    - from: "web/src/main.ts buildConfig()"
      to: "crates/puzzle-core/src/config.rs TabConfig"
      via: "JSON serialization with optional size_pct_max/taper_max fields"
      pattern: "size_pct_max"
    - from: "crates/puzzle-core/src/grid.rs generate_connectors()"
      to: "crates/puzzle-core/src/config.rs TabConfig"
      via: "reads size_pct_max/taper_max to pick random per-edge values"
      pattern: "random_range"
---

<objective>
Add per-edge randomization option for tab size and taper sliders.

Purpose: Allow each internal edge to have a unique tab size and/or taper value within a user-defined range, making puzzles more varied and natural-looking.
Output: Checkbox toggles next to tab size and taper sliders. When enabled, each slider becomes a dual-thumb range selector. Each edge gets a random value (seeded, deterministic) within the selected [min, max] range.
</objective>

<execution_context>
@/home/caleb/.config/Claude/get-shit-done/workflows/execute-plan.md
@/home/caleb/.config/Claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@crates/puzzle-core/src/config.rs
@crates/puzzle-core/src/grid.rs
@crates/puzzle-core/src/edge.rs
@crates/puzzle-core/src/classic_connector.rs
@crates/puzzle-core/src/connector.rs
@crates/puzzle-wasm/src/lib.rs
@web/index.html
@web/src/main.ts
@web/src/style.css

<interfaces>
From crates/puzzle-core/src/config.rs:
```rust
pub struct TabConfig {
    pub size_pct: f64,    // 0.15..=0.25
    pub taper: f64,       // 0.50..=1.20
}
impl TabConfig {
    pub fn validate(&self) -> Result<(), String>;
    pub fn neck_ratio(&self) -> f64;
}
```

From crates/puzzle-core/src/edge.rs:
```rust
pub struct EdgeParams {
    pub length: f64,
    pub direction: TabDirection,
    pub tab_size: f64,
    pub neck_ratio: f64,
}
```

From crates/puzzle-core/src/grid.rs generate_connectors():
```rust
// Currently uses single effective_tab_size and neck_ratio for ALL edges
let effective_tab_size = self.config.tab.size_pct.min(safe_max);
let neck_ratio = self.config.tab.neck_ratio();
```

From web/src/main.ts buildConfig():
```typescript
tab: { size_pct: parseFloat(tabSlider.value), taper: 0.5 + parseFloat(taperSlider.value) * 0.7 }
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add range fields to Rust TabConfig and per-edge randomization in generate_connectors</name>
  <files>
    crates/puzzle-core/src/config.rs
    crates/puzzle-core/src/grid.rs
    crates/puzzle-wasm/src/lib.rs
  </files>
  <action>
1. In `config.rs`, add two optional range fields to `TabConfig`:
   ```rust
   /// Optional max for per-edge randomization. When Some, each edge gets
   /// a random size_pct in [size_pct, size_pct_max] range.
   #[serde(default, skip_serializing_if = "Option::is_none")]
   pub size_pct_max: Option<f64>,
   /// Optional max for per-edge taper randomization. When Some, each edge gets
   /// a random taper in [taper, taper_max] range.
   #[serde(default, skip_serializing_if = "Option::is_none")]
   pub taper_max: Option<f64>,
   ```
   Update `Default` impl to set both to `None`.
   Update `validate()` to also validate the max values when present (same bounds as min: size_pct_max in 0.15..=0.25, taper_max in 0.50..=1.20, and max >= min).
   Add a helper method `fn randomize_tab_size(&self, safe_max: f64, rng: &mut ChaCha8Rng) -> f64` that returns either the fixed value (if size_pct_max is None) or a random value in [size_pct.min(safe_max), size_pct_max.min(safe_max)] using `rng.random_range()`. Import `rand::RngExt` in config.rs.
   Add a helper method `fn randomize_neck_ratio(&self, rng: &mut ChaCha8Rng) -> f64` that returns either the fixed neck_ratio (if taper_max is None) or computes neck_ratio from a random taper in [taper, taper_max].

   Note: Must add `use rand::RngExt;` and `use rand_chacha::ChaCha8Rng;` to config.rs.

2. In `grid.rs` `generate_connectors()`: Replace the current fixed `effective_tab_size` and `neck_ratio` with per-edge calls to the new randomize helpers. Change the edge iteration loops:
   ```rust
   // BEFORE (single value for all edges):
   let effective_tab_size = self.config.tab.size_pct.min(safe_max);
   let neck_ratio = self.config.tab.neck_ratio();
   // ... for edge in &mut self.h_edges { ... tab_size: effective_tab_size, neck_ratio, ... }

   // AFTER (per-edge randomization):
   let safe_max = self.safe_tab_max();
   // Remove the pre-computed effective_tab_size and neck_ratio lines.
   // Inside each edge loop iteration:
   for edge in &mut self.h_edges {
       if edge.is_border { continue; }
       let tab_size = self.config.tab.randomize_tab_size(safe_max, &mut rng);
       let neck_ratio = self.config.tab.randomize_neck_ratio(&mut rng);
       let params = EdgeParams { length: edge.length(), direction: edge.direction, tab_size, neck_ratio };
       let curves = connector.generate(&params, &mut rng);
       edge.connector = Some(curves);
   }
   // Same pattern for v_edges loop.
   ```
   IMPORTANT: When neither range field is set (None), this must consume zero RNG values so existing seeds produce identical output (backward compatibility). The randomize helpers must only call rng.random_range when the range field is Some.

3. In `lib.rs` (`safe_tab_max` function): Also clamp the new optional max fields if present:
   ```rust
   if let Some(ref mut max) = config.tab.size_pct_max {
       *max = max.clamp(0.15, 0.25);
   }
   if let Some(ref mut max) = config.tab.taper_max {
       *max = max.clamp(0.50, 1.20);
   }
   ```

4. Update existing tests in config.rs that construct TabConfig to include the new fields as None.
   Add a test that verifies when size_pct_max is None, the randomize method returns the fixed value without consuming RNG.
   Add a test that verifies when size_pct_max is Some, different values are returned for different calls.

5. Build WASM: Run `wasm-pack build crates/puzzle-wasm --target web --release` and copy the output to `web/pkg/`.
  </action>
  <verify>
    `cargo test --manifest-path crates/puzzle-core/Cargo.toml` passes (all existing + new tests).
    `cargo test --manifest-path crates/puzzle-wasm/Cargo.toml` passes.
    WASM builds successfully: `ls web/pkg/puzzle_wasm_bg.wasm` exists.
  </verify>
  <done>
    TabConfig has optional size_pct_max and taper_max fields. generate_connectors picks per-edge random values when ranges are set. When ranges are None, output is bit-identical to before (backward compat). WASM binary is rebuilt with new config support.
  </done>
</task>

<task type="auto">
  <name>Task 2: Add dual-range slider UI with checkbox toggles, URL sync, and styling</name>
  <files>
    web/index.html
    web/src/main.ts
    web/src/style.css
  </files>
  <action>
1. In `index.html`, modify the Tab Size and Taper slider groups to include:
   - A checkbox toggle in the slider-label row (between the label and readout)
   - A second range input for the max value (hidden by default)
   
   For Tab Size slider group, replace the current slider-group div with:
   ```html
   <div class="slider-group" id="tab-group">
     <div class="slider-label">
       <label for="tab">Tab Size</label>
       <label class="randomize-toggle" title="Randomize per edge">
         <input type="checkbox" id="tab-randomize" />
         <span class="toggle-icon">&#9860;</span>
       </label>
       <span class="readout" id="tab-readout">25%</span>
     </div>
     <div class="range-slider-container">
       <input type="range" id="tab" min="0.15" max="0.25" step="0.01" value="0.25" />
       <input type="range" id="tab-max" min="0.15" max="0.25" step="0.01" value="0.25" class="range-max" style="display:none" />
     </div>
   </div>
   ```
   
   Same pattern for Taper (ids: taper-randomize, taper, taper-max). The taper-max slider has same min/max/step as taper slider (min="0" max="1" step="0.01").
   
   The dual-range approach: When checkbox is checked, both range inputs are shown overlaid on each other. The first (min) input is constrained to not exceed the second (max), and vice versa. They share the same track visually.

2. In `style.css`, add styles for the randomize toggle and dual-range slider:
   ```css
   .randomize-toggle {
     display: flex;
     align-items: center;
     cursor: pointer;
     font-size: 0.85rem;
     color: #999;
     gap: 0.15rem;
   }
   .randomize-toggle input[type="checkbox"] {
     display: none;
   }
   .randomize-toggle input[type="checkbox"]:checked + .toggle-icon {
     color: #4a90d9;
   }
   .toggle-icon {
     font-size: 1rem;
     transition: color 0.15s;
   }
   .range-slider-container {
     position: relative;
     height: 1.5rem;
   }
   .range-slider-container input[type="range"] {
     position: absolute;
     width: 100%;
     top: 0;
     pointer-events: none;
     -webkit-appearance: none;
     appearance: none;
     background: transparent;
   }
   .range-slider-container input[type="range"]::-webkit-slider-thumb {
     pointer-events: all;
     -webkit-appearance: none;
     appearance: none;
     height: 16px;
     width: 16px;
     border-radius: 50%;
     background: #4a90d9;
     cursor: pointer;
     border: 2px solid #fff;
     box-shadow: 0 1px 3px rgba(0,0,0,0.2);
   }
   .range-slider-container input[type="range"]::-moz-range-thumb {
     pointer-events: all;
     height: 16px;
     width: 16px;
     border-radius: 50%;
     background: #4a90d9;
     cursor: pointer;
     border: 2px solid #fff;
     box-shadow: 0 1px 3px rgba(0,0,0,0.2);
   }
   /* Ensure only the first (min) range shows the track */
   .range-slider-container input[type="range"].range-max::-webkit-slider-runnable-track {
     background: transparent;
   }
   .range-slider-container input[type="range"].range-max::-moz-range-track {
     background: transparent;
   }
   /* Style the track for the min slider */
   .range-slider-container input[type="range"]:first-child::-webkit-slider-runnable-track {
     height: 4px;
     background: #ddd;
     border-radius: 2px;
   }
   .range-slider-container input[type="range"]:first-child::-moz-range-track {
     height: 4px;
     background: #ddd;
     border-radius: 2px;
   }
   ```
   Remove or override the existing `input[type="range"] { width: 100%; accent-color: #4a90d9; cursor: pointer; }` so it doesn't conflict. Move cursor/accent-color into the container styles as needed.

3. In `main.ts`:
   a) Add DOM references for the new elements:
      ```typescript
      let tabRandomize: HTMLInputElement;
      let tabMaxSlider: HTMLInputElement;
      let taperRandomize: HTMLInputElement;
      let taperMaxSlider: HTMLInputElement;
      ```
   
   b) Cache them in main():
      ```typescript
      tabRandomize = document.getElementById("tab-randomize") as HTMLInputElement;
      tabMaxSlider = document.getElementById("tab-max") as HTMLInputElement;
      taperRandomize = document.getElementById("taper-randomize") as HTMLInputElement;
      taperMaxSlider = document.getElementById("taper-max") as HTMLInputElement;
      ```
   
   c) Checkbox toggle logic: When checked, show the max slider and update readout to show "min-max" format. When unchecked, hide max slider and revert readout.
      ```typescript
      function toggleRandomize(checkbox: HTMLInputElement, maxSlider: HTMLInputElement, minSlider: HTMLInputElement): void {
        if (checkbox.checked) {
          maxSlider.style.display = '';
          // Ensure max >= min
          if (parseFloat(maxSlider.value) < parseFloat(minSlider.value)) {
            maxSlider.value = minSlider.value;
          }
        } else {
          maxSlider.style.display = 'none';
        }
        updateReadouts();
        generatePuzzle();
      }
      ```
   
   d) Constrain min/max: On min slider input, clamp max to be >= min. On max slider input, clamp min to be <= max.
      ```typescript
      function clampRange(minSlider: HTMLInputElement, maxSlider: HTMLInputElement): void {
        const minVal = parseFloat(minSlider.value);
        const maxVal = parseFloat(maxSlider.value);
        if (maxVal < minVal) {
          maxSlider.value = minSlider.value;
        }
      }
      ```
      Similarly for the reverse direction.
   
   e) Update `buildConfig()` to include optional max values:
      ```typescript
      const tabConfig: Record<string, unknown> = {
        size_pct: parseFloat(tabSlider.value),
        taper: 0.5 + parseFloat(taperSlider.value) * 0.7
      };
      if (tabRandomize.checked) {
        tabConfig.size_pct_max = parseFloat(tabMaxSlider.value);
      }
      if (taperRandomize.checked) {
        tabConfig.taper_max = 0.5 + parseFloat(taperMaxSlider.value) * 0.7;
      }
      return { ...otherFields, tab: tabConfig };
      ```
   
   f) Update `updateReadouts()` to show range format when randomize is on:
      - Tab: "15%-25%" instead of "25%"
      - Taper: "0.20-0.80" instead of "0.50"
   
   g) Update `updateTabMax()` to also set max on the tab-max slider.
   
   h) Update `loadFromURL()` to restore randomize checkboxes and max values:
      - New URL params: `tabr=1` (tab randomize on), `tabmax=25` (tab max as int pct), `taperr=1` (taper randomize on), `tapermax=80` (taper max as int pct)
   
   i) Update `updateURL()` to serialize the randomize state and max values (only when checkbox is on).
   
   j) Wire events:
      - tabRandomize change → toggleRandomize
      - taperRandomize change → toggleRandomize
      - tabMaxSlider input → clampRange + updateReadouts + generatePuzzle
      - taperMaxSlider input → clampRange + updateReadouts + generatePuzzle
      - tabSlider input → also clampRange(tabSlider, tabMaxSlider) when randomize is on
      - taperSlider input → also clampRange(taperSlider, taperMaxSlider) when randomize is on
      - Add tabMaxSlider and taperMaxSlider to the slider event listeners for updateTabMax recalculation
  </action>
  <verify>
    Run `npm run build` (or `npx vite build`) from web/ directory — no TypeScript errors.
    Run `npx vite --host` from web/ and visually verify:
    1. Tab Size and Taper sliders show dice icon checkbox
    2. Clicking checkbox reveals second thumb on slider
    3. Moving thumbs shows "min%-max%" in readout
    4. Puzzle regenerates with visible variation in tab sizes across edges
    5. Copy Link includes randomize params, pasting URL restores state
  </verify>
  <done>
    Dual-range sliders work for tab size and taper. Checkboxes toggle between single-value and range mode. Readouts show "min-max" when in range mode. URL sharing preserves all randomize state. Each edge visually has different tab sizes/tapers when randomize is enabled.
  </done>
</task>

</tasks>

<verification>
1. `cargo test --workspace` — all Rust tests pass
2. `npm run build` in web/ — no build errors
3. With randomize OFF: same seed produces identical SVG as before (backward compat)
4. With randomize ON: edges have visibly different tab sizes; same seed still deterministic
5. URL round-trip: copy link with randomize on → paste → same puzzle renders
</verification>

<success_criteria>
- Tab size and taper sliders each have a randomize checkbox toggle
- When enabled, slider shows dual thumbs for min/max range selection
- Each internal edge gets a seeded-random value within the selected range
- Backward compatibility: puzzles without randomize produce identical output
- URL sharing preserves randomize mode and range values
</success_criteria>

<output>
After completion, create `.planning/quick/5-add-randomize-per-edge-option-to-tab-siz/5-SUMMARY.md`
</output>
