---
phase: quick-019
plan: 1
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/puzzle-core/src/config.rs
  - crates/puzzle-core/src/edge.rs
  - crates/puzzle-core/src/classic_connector.rs
  - crates/puzzle-core/src/grid.rs
  - web/index.html
  - web/src/main.ts
autonomous: true
requirements: [QUICK-019]
must_haves:
  truths:
    - "Tab offset slider visible in Parameters section of controls panel"
    - "Moving the slider shifts the knob position along the edge away from center"
    - "Offset 0 keeps knob centered (default, backward compatible)"
    - "Offset is per-edge randomizable like tab size and taper"
    - "URL sharing preserves offset value"
  artifacts:
    - path: "crates/puzzle-core/src/config.rs"
      provides: "TabConfig.offset field with validation and randomize_offset()"
    - path: "crates/puzzle-core/src/edge.rs"
      provides: "EdgeParams.offset field"
    - path: "crates/puzzle-core/src/classic_connector.rs"
      provides: "Offset-shifted center calculation in generate()"
    - path: "web/index.html"
      provides: "Tab Offset slider UI with randomize toggle"
    - path: "web/src/main.ts"
      provides: "Offset slider wiring, buildConfig, URL sync"
  key_links:
    - from: "web/src/main.ts buildConfig()"
      to: "WASM generate_edges_binary"
      via: "tab.offset JSON field"
      pattern: "offset"
    - from: "crates/puzzle-core/src/grid.rs generate_connectors()"
      to: "EdgeParams"
      via: "config.tab.randomize_offset()"
      pattern: "randomize_offset"
    - from: "crates/puzzle-core/src/classic_connector.rs"
      to: "center calculation"
      via: "params.offset shifts center from 0.5"
      pattern: "center.*offset"
---

<objective>
Add a "Tab Offset" slider to the Parameters section that shifts the tab/knob position slightly off-center along each edge. Offset 0 = centered (default, backward compatible). The slider supports per-edge randomization like tab size and taper.

Purpose: More natural-looking puzzles where tabs aren't perfectly centered on every edge.
Output: Working offset slider with Rust config, WASM plumbing, and full UI integration.
</objective>

<execution_context>
@.planning/quick/19-add-tab-offset-slider-that-moves-the-tab/19-PLAN.md
</execution_context>

<context>
@.planning/STATE.md
@crates/puzzle-core/src/config.rs
@crates/puzzle-core/src/edge.rs
@crates/puzzle-core/src/classic_connector.rs
@crates/puzzle-core/src/grid.rs
@web/index.html
@web/src/main.ts

<interfaces>
From crates/puzzle-core/src/config.rs:
```rust
pub struct TabConfig {
    pub size_pct: f64,        // 0.15..=0.25
    pub taper: f64,           // 0.57..=1.32
    pub size_pct_max: Option<f64>,
    pub taper_max: Option<f64>,
}
// Methods: validate(), neck_ratio(), randomize_tab_size(), randomize_neck_ratio()
```

From crates/puzzle-core/src/edge.rs:
```rust
pub struct EdgeParams {
    pub length: f64,
    pub cross_length: f64,
    pub direction: TabDirection,
    pub tab_size: f64,
    pub neck_ratio: f64,
}
```

From crates/puzzle-core/src/classic_connector.rs:
```rust
// Line 71: center is always length * 0.5
let center = length * 0.5;
// All knob geometry references `center` for X positioning
```

From crates/puzzle-core/src/grid.rs generate_connectors():
```rust
// For each internal edge, constructs EdgeParams with:
let tab_size = self.config.tab.randomize_tab_size(safe_max, &mut rng);
let neck_ratio = self.config.tab.randomize_neck_ratio(&mut rng);
let params = EdgeParams { length, cross_length, direction, tab_size, neck_ratio };
```

From web/src/main.ts buildConfig():
```typescript
function buildConfig(): object {
  const tabConfig = { size_pct, taper, size_pct_max?, taper_max? };
  return { rows, cols, width, height, unit, tab: tabConfig, seed };
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add offset field to Rust config, edge params, and connector generator</name>
  <files>
    crates/puzzle-core/src/config.rs
    crates/puzzle-core/src/edge.rs
    crates/puzzle-core/src/classic_connector.rs
    crates/puzzle-core/src/grid.rs
  </files>
  <action>
**config.rs — Add offset to TabConfig:**
- Add `pub offset: f64` field with `#[serde(default)]` — represents how far the knob center shifts from the midpoint as a fraction of edge length. Range: -0.15..=0.15 (negative = shift left, positive = shift right). Default 0.0.
- Add `pub offset_max: Option<f64>` with `#[serde(default, skip_serializing_if = "Option::is_none")]` — for per-edge randomization.
- In `Default for TabConfig`: set `offset: 0.0, offset_max: None`.
- In `validate()`: check offset is in -0.15..=0.15. If offset_max is Some, validate it's in -0.15..=0.15 and that the range [offset, offset_max] is valid (max >= offset, i.e. the "max" is the upper end). Actually, since offset can be negative and we want a range, treat it as: offset is the low end, offset_max is the high end; random picks from [offset..=offset_max]. Both must be in -0.15..=0.15 and offset_max >= offset.
- Add `pub fn randomize_offset(&self, rng: &mut ChaCha8Rng) -> f64` — follows same pattern as randomize_tab_size: when offset_max is None, return self.offset without consuming RNG. When Some, return rng.random_range(self.offset..=max).

**edge.rs — Add offset to EdgeParams:**
- Add `pub offset: f64` field to EdgeParams struct. This is the already-randomized offset for this specific edge.

**classic_connector.rs — Use offset in center calculation:**
- Change line 71 from `let center = length * 0.5;` to `let center = length * (0.5 + params.offset);`
- This shifts the entire knob along the edge. With offset=0 behavior is identical to current (backward compatible). With offset=0.1, knob shifts 10% of edge length to the right.
- The approach curve start/end points (p0 at 0.0, p3 at length) remain fixed — only the knob center moves. This means the first bezier will have a longer/shorter approach on one side.

**grid.rs — Pass offset to EdgeParams in generate_connectors():**
- In both h_edges and v_edges loops, after randomize_tab_size and randomize_neck_ratio, add:
  `let offset = self.config.tab.randomize_offset(&mut rng);`
- Add `offset` to the EdgeParams construction.

**Important: RNG consumption order** — offset randomization MUST happen AFTER tab_size and neck_ratio randomization to preserve backward compatibility when offset=0 (None). The randomize_offset() method consumes zero RNG when offset_max is None.
  </action>
  <verify>
    Run `cargo test --workspace` from project root. All existing tests pass. The default offset=0 produces identical output.
    Run `cargo build --release --target wasm32-unknown-unknown -p puzzle-wasm` to verify WASM compiles.
  </verify>
  <done>
    - TabConfig has offset and offset_max fields with validation
    - EdgeParams has offset field
    - ClassicKnobConnector uses offset to shift knob center
    - Grid passes randomized offset to EdgeParams
    - All existing tests pass (backward compatible — offset defaults to 0)
    - WASM compiles successfully
  </done>
</task>

<task type="auto">
  <name>Task 2: Add Tab Offset slider to UI with randomize toggle and URL sync</name>
  <files>
    web/index.html
    web/src/main.ts
  </files>
  <action>
**index.html — Add offset slider group in Parameters section:**
After the taper slider group (after `</div>` closing `id="taper-group"`), add a new slider group following the exact same pattern as taper:

```html
<div class="slider-group" id="offset-group">
  <div class="slider-label">
    <label for="offset">Tab Offset</label>
    <label class="randomize-toggle" title="Randomize per edge">
      <input type="checkbox" id="offset-randomize" />
      <span class="toggle-icon">&#9860;</span>
    </label>
    <span class="readout" id="offset-readout">0</span>
  </div>
  <div class="range-slider-container" id="offset-track">
    <input type="range" id="offset" min="-0.15" max="0.15" step="0.01" value="0" />
    <input type="range" id="offset-max" min="-0.15" max="0.15" step="0.01" value="0.15" class="range-max" style="display:none" />
  </div>
</div>
```

**main.ts — Wire up offset slider:**

1. **DOM references** — Add at top near other slider refs:
   - `let offsetSlider: HTMLInputElement;`
   - `let offsetReadout: HTMLElement;`
   - `let offsetRandomize: HTMLInputElement;`
   - `let offsetMaxSlider: HTMLInputElement;`
   - `let offsetTrack: HTMLElement;`

2. **Cache DOM** in main() after taperTrack:
   - `offsetSlider = document.getElementById("offset") as HTMLInputElement;`
   - `offsetReadout = document.getElementById("offset-readout")!;`
   - `offsetRandomize = document.getElementById("offset-randomize") as HTMLInputElement;`
   - `offsetMaxSlider = document.getElementById("offset-max") as HTMLInputElement;`
   - `offsetTrack = document.getElementById("offset-track")!;`

3. **buildConfig()** — Add offset to tabConfig:
   ```typescript
   tabConfig.offset = parseFloat(offsetSlider.value);
   if (offsetRandomize.checked) {
     tabConfig.offset_max = parseFloat(offsetMaxSlider.value);
   }
   ```

4. **updateReadouts()** — Add offset readout update:
   ```typescript
   if (offsetRandomize.checked) {
     offsetReadout.textContent = `${parseFloat(offsetSlider.value).toFixed(2)}-${parseFloat(offsetMaxSlider.value).toFixed(2)}`;
   } else {
     offsetReadout.textContent = parseFloat(offsetSlider.value).toFixed(2);
   }
   updateRangeHighlight(offsetSlider, offsetMaxSlider, offsetTrack, offsetRandomize.checked);
   ```

5. **loadFromURL()** — Add offset URL param restore (after taper restore):
   ```typescript
   const offsetVal = parseInt(params.get("off") ?? "0", 10) / 100;
   const offset = Math.max(-0.15, Math.min(0.15, offsetVal));
   offsetSlider.value = String(offset);
   if (params.get("offr") === "1") {
     offsetRandomize.checked = true;
     const offMax = Math.max(-0.15, Math.min(0.15, parseInt(params.get("offmax") ?? "15", 10) / 100));
     offsetMaxSlider.value = String(offMax);
     offsetMaxSlider.style.display = "";
   }
   ```

6. **updateURL()** — Add offset URL params:
   ```typescript
   params.set("off", String(Math.round(parseFloat(offsetSlider.value) * 100)));
   if (offsetRandomize.checked) {
     params.set("offr", "1");
     params.set("offmax", String(Math.round(parseFloat(offsetMaxSlider.value) * 100)));
   }
   ```

7. **Event wiring** in main() — Add offset slider to the `sliders` array that gets input listeners:
   - Add `offsetSlider` to the `const sliders = [tabSlider, taperSlider]` → `[tabSlider, taperSlider, offsetSlider]`
   - Add offset-specific randomize clamping in the slider input handler:
     ```typescript
     if (slider === offsetSlider && offsetRandomize.checked) {
       clampMinMax(offsetSlider, offsetMaxSlider);
     }
     ```
   - Add offset max slider event (same pattern as taperMaxSlider):
     ```typescript
     offsetMaxSlider.addEventListener("input", () => {
       clampMaxMin(offsetSlider, offsetMaxSlider);
       updateReadouts();
       scheduleGenerate();
     });
     ```
   - Add offset randomize checkbox event:
     ```typescript
     offsetRandomize.addEventListener("change", () => {
       toggleRandomize(offsetRandomize, offsetMaxSlider, offsetSlider);
     });
     ```

8. **Build WASM** — After all code changes, rebuild WASM:
   ```bash
   wasm-pack build crates/puzzle-wasm --target web --out-dir ../../web/pkg
   ```
  </action>
  <verify>
    Run `npm run dev` from `web/` directory. Verify:
    1. Tab Offset slider appears in Parameters section below Taper
    2. Default value is 0 — puzzle looks identical to before
    3. Moving slider left/right visibly shifts knob positions along edges
    4. Randomize toggle works — checkbox enables max slider, readout shows range
    5. URL includes `off=` param when non-zero
    6. Reload with URL params restores offset state
  </verify>
  <done>
    - Tab Offset slider visible in UI with range -0.15 to 0.15, default 0
    - Randomize per-edge toggle works with center-aware behavior
    - buildConfig sends offset to WASM
    - URL sync preserves offset value (off= param, integer percentage)
    - WASM rebuilt and puzzle renders with offset applied
  </done>
</task>

</tasks>

<verification>
1. `cargo test --workspace` — all Rust tests pass
2. Open browser, verify default puzzle unchanged (offset=0)
3. Move offset slider — knobs visibly shift along edges
4. Enable randomize — each edge gets different offset
5. Copy link, open in new tab — offset state preserved
6. Download SVG — offset reflected in downloaded file
</verification>

<success_criteria>
- Tab Offset slider in Parameters section with -0.15 to 0.15 range
- Default 0 produces identical output to before (backward compatible)
- Non-zero offset visibly shifts knobs off-center
- Per-edge randomization via toggle checkbox
- URL sharing preserves offset
- All existing tests pass
</success_criteria>

<output>
After completion, create `.planning/quick/19-add-tab-offset-slider-that-moves-the-tab/19-SUMMARY.md`
</output>
