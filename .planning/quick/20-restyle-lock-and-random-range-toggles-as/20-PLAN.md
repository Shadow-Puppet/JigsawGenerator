---
phase: 20-restyle-toggles
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - web/index.html
  - web/src/style.css
  - web/src/main.ts
autonomous: true
requirements: [TOGGLE-RESTYLE, TAB-MAX-20]
must_haves:
  truths:
    - "Lock toggles display as pill-shaped toggle switches with monochrome lock/unlock icon on left"
    - "Random range toggles display as pill-shaped toggle switches with die icon on left"
    - "Random range toggle icon appears right after the parameter name (not between label and readout)"
    - "Max tab size is 20% (was 25%)"
    - "Tab offset max still scales properly with tab size"
    - "All toggle states sync correctly with URL params and input behavior"
  artifacts:
    - path: "web/index.html"
      provides: "Toggle switch HTML structure"
    - path: "web/src/style.css"
      provides: "Pill toggle switch styles"
    - path: "web/src/main.ts"
      provides: "Tab max cap at 0.20, toggle behavior"
  key_links:
    - from: "web/index.html"
      to: "web/src/style.css"
      via: "CSS class names for toggle switches"
    - from: "web/src/main.ts"
      to: "web/index.html"
      via: "DOM IDs for lock and randomize toggles"
---

<objective>
Restyle lock and random-range toggles from plain icons to pill-shaped toggle switches (sliding circle indicator) with monochrome icons. Move random-range icons to appear right after parameter names. Change max tab size cap from 25% to 20% and ensure offset scaling still works.

Purpose: Better visual clarity for toggle states and cleaner parameter layout
Output: Updated HTML/CSS/TS with pill toggle switches and 20% tab max
</objective>

<execution_context>
@.planning/quick/20-restyle-lock-and-random-range-toggles-as/20-PLAN.md
</execution_context>

<context>
@web/index.html
@web/src/style.css
@web/src/main.ts
</context>

<tasks>

<task type="auto">
  <name>Task 1: Restyle lock and randomize toggles as pill switches and cap tab max at 20%</name>
  <files>web/index.html, web/src/style.css, web/src/main.ts</files>
  <action>
**HTML changes (index.html):**

1. Replace lock toggle buttons in section headers with pill-switch structure. For both `#grid-lock` and `#dims-lock`, change from:
   ```html
   <button type="button" class="lock-toggle" id="grid-lock" title="Lock grid size">&#128275;</button>
   ```
   To a label+checkbox pattern with a pill track and sliding circle:
   ```html
   <label class="pill-toggle" title="Lock grid size">
     <span class="pill-icon">&#x1F513;</span>
     <input type="checkbox" id="grid-lock" />
     <span class="pill-track"><span class="pill-knob"></span></span>
   </label>
   ```
   Use Unicode &#x1F513; (open lock) as the default icon. The icon will be swapped to &#x1F512; (closed lock) in JS when checked. Same pattern for `#dims-lock`.

2. Replace randomize toggles. For tab-randomize and taper-randomize, change the current structure to a pill-toggle, and move the toggle from between label and readout to right after the label text. Current:
   ```html
   <div class="slider-label">
     <label for="tab">Tab Size</label>
     <label class="randomize-toggle" title="Randomize per edge">
       <input type="checkbox" id="tab-randomize" />
       <span class="toggle-icon">&#9860;</span>
     </label>
     <span class="readout" id="tab-readout">25%</span>
   </div>
   ```
   Change to:
   ```html
   <div class="slider-label">
     <span class="label-with-toggle">
       <label for="tab">Tab Size</label>
       <label class="pill-toggle pill-toggle-sm" title="Randomize per edge">
         <span class="pill-icon">&#9860;</span>
         <input type="checkbox" id="tab-randomize" />
         <span class="pill-track"><span class="pill-knob"></span></span>
       </label>
     </span>
     <span class="readout" id="tab-readout">20%</span>
   </div>
   ```
   Do same for taper-randomize. Note the default readout should be "20%" not "25%" since we're lowering the cap.

**CSS changes (style.css):**

1. Remove old `.lock-toggle` and `.randomize-toggle` / `.toggle-icon` styles entirely.

2. Add new `.pill-toggle` styles:
   - `.pill-toggle`: `display: inline-flex; align-items: center; gap: 4px; cursor: pointer; user-select: none;`
   - `.pill-toggle input[type="checkbox"]`: `display: none;`
   - `.pill-icon`: `font-size: 0.75rem; color: #999; line-height: 1; filter: grayscale(1); transition: filter 0.15s;` — the grayscale filter makes lock emojis monochrome
   - `.pill-toggle input:checked ~ .pill-icon` should NOT exist (icon is before input, so use JS or adjacent sibling differently). Instead, use `.pill-toggle.active .pill-icon { filter: grayscale(0); }` — NO, keep it monochrome per user request. So `.pill-icon` always has `filter: grayscale(1); opacity: 0.5;` and when active: `.pill-toggle.active .pill-icon { opacity: 1; }` (still grayscale but more visible).
   
   Actually, simplify: The icon sits left of the track, and we manage state via `.active` class toggled by JS:
   - `.pill-icon`: `font-size: 0.7rem; opacity: 0.4; filter: grayscale(1); transition: opacity 0.15s;`
   - `.pill-toggle.active .pill-icon`: `opacity: 0.8;`

3. `.pill-track`: The pill shape track.
   ```css
   .pill-track {
     position: relative;
     width: 26px;
     height: 14px;
     background: #ccc;
     border-radius: 7px;
     transition: background 0.2s;
   }
   .pill-toggle.active .pill-track {
     background: #4a90d9;
   }
   ```

4. `.pill-knob`: The sliding circle.
   ```css
   .pill-knob {
     position: absolute;
     top: 2px;
     left: 2px;
     width: 10px;
     height: 10px;
     background: #fff;
     border-radius: 50%;
     transition: left 0.2s;
   }
   .pill-toggle.active .pill-knob {
     left: 14px;
   }
   ```

5. `.pill-toggle-sm`: Smaller variant for the randomize toggles next to parameter names.
   ```css
   .pill-toggle-sm .pill-track {
     width: 22px;
     height: 12px;
     border-radius: 6px;
   }
   .pill-toggle-sm .pill-knob {
     width: 8px;
     height: 8px;
     top: 2px;
     left: 2px;
   }
   .pill-toggle-sm.active .pill-knob {
     left: 12px;
   }
   ```

6. `.label-with-toggle`: `display: inline-flex; align-items: center; gap: 6px;` — groups the label text and its randomize toggle together on the left side of the slider-label row, with readout pushed to the right via the existing `justify-content: space-between` on `.slider-label`.

**JS changes (main.ts):**

1. Change lock toggle variables from `HTMLElement` to proper types. The `gridLockBtn` and `dimsLockBtn` are now `<label>` wrappers, but the actual state is driven by the `<input type="checkbox">` inside. Change to track checkbox elements:
   - Add new refs: `gridLockCheckbox` and `dimsLockCheckbox` (HTMLInputElement)
   - Cache them: `gridLockCheckbox = document.getElementById('grid-lock') as HTMLInputElement;`
   - The parent `.pill-toggle` label is `gridLockCheckbox.closest('.pill-toggle')`.

2. Rewrite `toggleLock()` function. Remove old implementation that used innerHTML emoji swapping and `.locked` class. Instead:
   ```typescript
   function toggleLock(checkbox: HTMLInputElement, label: string): boolean {
     const active = checkbox.checked;
     const pill = checkbox.closest('.pill-toggle')!;
     pill.classList.toggle('active', active);
     // Update icon: 🔒 when locked, 🔓 when unlocked
     const icon = pill.querySelector('.pill-icon')!;
     icon.innerHTML = active ? '&#x1F512;' : '&#x1F513;';
     pill.setAttribute('title', active ? `Unlock ${label}` : `Lock ${label}`);
     return active;
   }
   ```

3. Update lock event listeners to use checkbox change events:
   ```typescript
   gridLockCheckbox.addEventListener('change', () => {
     gridLocked = toggleLock(gridLockCheckbox, 'grid size');
   });
   dimsLockCheckbox.addEventListener('change', () => {
     dimsLocked = toggleLock(dimsLockCheckbox, 'dimensions');
   });
   ```

4. Update randomize toggle handling. The existing `tabRandomize` and `taperRandomize` inputs remain as checkbox inputs with the same IDs, so `toggleRandomize()` still works. But add `.active` class toggling on the parent `.pill-toggle`:
   - In `toggleRandomize()`, after checkbox check, add:
     ```typescript
     const pill = checkbox.closest('.pill-toggle');
     if (pill) pill.classList.toggle('active', checkbox.checked);
     ```

5. **Change max tab size from 0.25 to 0.20:**
   - In `updateTabMax()`: change `const tabMax = Math.min(safeMax, 0.25);` → `const tabMax = Math.min(safeMax, 0.20);`
   - In `loadFromURL()`: change the tab clamping line:
     `const tab = Math.max(0.15, Math.min(0.25, ...))` → `Math.max(0.15, Math.min(0.20, ...))`
     Also change tabMax restoration: `Math.max(0.15, Math.min(0.25, ...))` → `Math.max(0.15, Math.min(0.20, ...))`

6. Update HTML slider max attributes to match: in index.html, change `#tab` and `#tab-max` sliders' `max="0.25"` to `max="0.20"`, and their default `value="0.25"` to `value="0.20"`.

7. **Verify offset scaling:** The `updateOffsetMax()` formula `0.35 - tabSize` still works correctly. At 20% tab → max offset = 0.15. At 15% tab → max offset = 0.20. The formula is tab-size-dependent, not cap-dependent, so no changes needed there. But update `loadFromURL()` offset clamp from `Math.min(0.20, ...)` to `Math.min(0.20, ...)` — actually check: current is `Math.max(0, Math.min(0.20, offsetVal))`. With new 20% max tab and formula `0.35 - 0.20 = 0.15`, the max possible offset is now 0.15 (not 0.20). But the loadFromURL hardcoded clamp at 0.20 is just a safety bound before `updateOffsetMax` runs. This is fine as-is since `updateOffsetMax()` will further clamp it after load. No change needed.

8. Also update the readout handling: In the `loadFromURL` function where randomize toggles are restored, add `.active` class to the pill-toggle parent when the checkbox is checked:
   ```typescript
   if (params.get("tabr") === "1") {
     tabRandomize.checked = true;
     tabRandomize.closest('.pill-toggle')?.classList.add('active');
     // ... existing code
   }
   ```
   Same for taper randomize.
  </action>
  <verify>
    Run `npm run build` from `web/` directory to verify TypeScript compiles and Vite builds without errors.
    Then run `npm run dev` and visually confirm:
    - Lock toggles show as pill switches with monochrome lock icons
    - Randomize toggles show as small pill switches right after "Tab Size" and "Taper" labels
    - Tab Size slider max is 20% 
    - Toggling any pill switch shows sliding animation and state change
    - URL params restore toggle states correctly on page reload
  </verify>
  <done>
    All toggles restyled as pill switches with monochrome icons. Tab max capped at 20%. Offset scaling works with new cap. Toggle states sync with URL params and function correctly.
  </done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <what-built>Restyled lock and randomize toggles as pill switches, reduced tab max to 20%</what-built>
  <how-to-verify>
    1. Run `npm run dev` in `web/` and open the app
    2. Verify lock toggles (Grid Size, Dimensions headers) show as pill-shaped switches with monochrome lock icon on left
    3. Click lock toggles — circle should slide and icon should change between open/closed lock
    4. Verify randomize toggles appear as small pill switches immediately after "Tab Size" and "Taper" text
    5. Toggle randomize — pill should slide, dual range sliders should appear/disappear
    6. Verify Tab Size slider maxes out at 20%
    7. With tab at 20%, verify Tab Offset slider max is ~0.15 (proper scaling)
    8. Copy link, reload — verify all toggle states restore correctly
  </how-to-verify>
  <resume-signal>Type "approved" or describe issues</resume-signal>
</task>

</tasks>

<verification>
- `npm run build` succeeds in web/ directory
- All toggle switches render as pill-shaped with sliding knob
- Lock icons are monochrome (grayscale filter applied)
- Randomize icons positioned right after parameter names
- Tab size max is 20%, offset max scales accordingly
- URL param round-trip preserves all toggle states
</verification>

<success_criteria>
- Pill-shaped toggle switches for both lock and randomize controls
- Monochrome lock/unlock icons (not colored)
- Random range icon positioned directly after parameter name label
- Tab size max capped at 20%
- Tab offset max correctly computed as 0.35 - tab_size
- No regressions in existing parameter behavior
</success_criteria>

<output>
After completion, create `.planning/quick/20-restyle-lock-and-random-range-toggles-as/20-SUMMARY.md`
</output>
