---
phase: quick-008
plan: 1
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/puzzle-core/src/svg_export.rs
autonomous: true
requirements: [QUICK-008]

must_haves:
  truths:
    - "Kerf=0 produces identical SVG to before (no regression)"
    - "Kerf>0 offsets only the border outward, not internal connector edges"
    - "With kerf=0.2mm, border bounding box is ~0.2mm larger in each dimension than kerf=0"
    - "Internal connector paths are unchanged regardless of kerf value"
  artifacts:
    - path: "crates/puzzle-core/src/svg_export.rs"
      provides: "Separate kerf handling for border vs internal edges"
    - path: "crates/puzzle-core/src/kerf.rs"
      provides: "Path offset utility (unchanged or simplified)"
  key_links:
    - from: "svg_export.rs::generate_svg"
      to: "kerf.rs::offset_path"
      via: "Apply offset to border subpath only"
      pattern: "offset_path.*border"
---

<objective>
Fix kerf compensation to only offset the border path, leaving internal connector edges unchanged.

Purpose: The current implementation applies a polyline offset to ALL paths (border + connectors) uniformly using left-side normals. This distorts connector geometry because: (1) connectors are open subpaths where "left side" has no meaningful outward direction, (2) tab curves that go in/out get uniformly shifted in one direction creating lopsided geometry, (3) for laser-cut jigsaws, the kerf on internal edges actually creates desirable clearance between pieces — only the border needs compensation to maintain correct overall dimensions.

Output: Working kerf compensation that expands the border by kerf_width/2 outward while preserving all internal connector geometry exactly.
</objective>

<execution_context>
@.planning/quick/8-investigate-and-fix-kerf-width-setting-c/8-PLAN.md
</execution_context>

<context>
@crates/puzzle-core/src/kerf.rs
@crates/puzzle-core/src/svg_export.rs
@crates/puzzle-core/src/config.rs

<interfaces>
From crates/puzzle-core/src/kerf.rs:
```rust
/// Offset all paths outward by `kerf_width / 2.0` for kerf compensation.
/// Flattens curves to polylines, offsets along left-side normals, miter joins.
pub fn offset_path(path: &BezPath, kerf_width: f64) -> BezPath;
```

From crates/puzzle-core/src/svg_export.rs:
```rust
/// Build the puzzle BezPath: closed border subpath + open internal edge subpaths
fn build_puzzle_path(grid: &PuzzleGrid) -> BezPath;

/// Apply kerf then render to SVG string
pub fn generate_svg(grid: &PuzzleGrid) -> String;
```

Current flow in generate_svg():
```rust
let mut path = build_puzzle_path(grid);       // border + connectors in one BezPath
if grid.config.kerf_width > 0.0 {
    path = offset_path(&path, grid.config.kerf_width);  // offsets EVERYTHING
}
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Separate border and connector path construction, apply kerf to border only</name>
  <files>crates/puzzle-core/src/svg_export.rs</files>
  <action>
Refactor `generate_svg()` and `build_puzzle_path()` to handle kerf correctly:

1. Split `build_puzzle_path()` into two functions:
   - `build_border_path(grid) -> BezPath` — extracts the existing border construction code (the "Group 1" section: move_to, line_to, quarter arcs, close_path)
   - `build_connector_paths(grid) -> BezPath` — extracts the existing internal edges code (the "Group 2" section: h_edges and v_edges iteration)
   
   Keep the original `build_puzzle_path()` as a thin wrapper that calls both and combines them (for backward compat with tests).

2. Update `generate_svg()`:
   ```rust
   pub fn generate_svg(grid: &PuzzleGrid) -> String {
       let mut border = build_border_path(grid);
       let connectors = build_connector_paths(grid);
       
       if grid.config.kerf_width > 0.0 {
           border = offset_path(&border, grid.config.kerf_width);
       }
       
       // Combine: border first, then connectors
       let mut combined = border;
       for el in connectors.iter() {
           match el {
               PathEl::MoveTo(p) => combined.move_to(p),
               PathEl::LineTo(p) => combined.line_to(p),
               PathEl::CurveTo(p1, p2, p3) => combined.curve_to(p1, p2, p3),
               PathEl::ClosePath => combined.close_path(),
               _ => {}
           }
       }
       
       let path_data = combined.to_svg();
       build_svg_document(&path_data, grid.config.width, grid.config.height)
   }
   ```

3. Add `use kurbo::PathEl;` to imports if not already present.

Do NOT change `kerf.rs` — the offset algorithm itself is fine for offsetting a closed border path. The bug is that it was being applied to everything.
  </action>
  <verify>
  Run `cargo test -p puzzle-core` — all existing tests must pass. The existing tests use kerf_width=0.0, so they validate no regression. The kerf-specific tests in kerf.rs test the offset algorithm in isolation (square paths) which remains correct.
  </verify>
  <done>generate_svg() applies kerf offset to border only; connector paths pass through unchanged; all existing tests pass.</done>
</task>

<task type="auto">
  <name>Task 2: Add targeted test proving kerf only affects border dimensions</name>
  <files>crates/puzzle-core/src/svg_export.rs</files>
  <action>
Add a new test to the `mod tests` block in `svg_export.rs` that validates kerf compensation works correctly:

```rust
#[test]
fn test_kerf_only_offsets_border() {
    let mut config_no_kerf = test_config(3, 4, "kerf-test");
    config_no_kerf.kerf_width = 0.0;
    let mut grid_no_kerf = PuzzleGrid::new(config_no_kerf).unwrap();
    grid_no_kerf.generate_connectors(&ClassicKnobConnector);
    let svg_no_kerf = generate_svg(&grid_no_kerf);

    let mut config_kerf = test_config(3, 4, "kerf-test");
    config_kerf.kerf_width = 0.2;
    let mut grid_kerf = PuzzleGrid::new(config_kerf).unwrap();
    grid_kerf.generate_connectors(&ClassicKnobConnector);
    let svg_kerf = generate_svg(&grid_kerf);

    // SVGs should differ (border is offset)
    assert_ne!(svg_no_kerf, svg_kerf, "kerf should change the SVG output");

    // Extract path data from both
    let extract_path = |svg: &str| -> String {
        let start = svg.find("d='").unwrap() + 3;
        let end = svg[start..].find('\'').unwrap() + start;
        svg[start..end].to_string()
    };

    let path_no_kerf = extract_path(&svg_no_kerf);
    let path_kerf = extract_path(&svg_kerf);

    // Count M commands — should be same count (same structure)
    let m_count_no_kerf = path_no_kerf.matches('M').count();
    let m_count_kerf = path_kerf.matches('M').count();
    assert_eq!(
        m_count_no_kerf, m_count_kerf,
        "kerf should not change number of subpaths"
    );

    // Internal connector curves (C commands after the first Z which ends the border)
    // should be identical between kerf and no-kerf
    let after_border = |path: &str| -> &str {
        if let Some(z_pos) = path.find('Z') {
            &path[z_pos + 1..]
        } else {
            ""
        }
    };

    let connectors_no_kerf = after_border(&path_no_kerf);
    let connectors_kerf = after_border(&path_kerf);
    assert_eq!(
        connectors_no_kerf, connectors_kerf,
        "kerf should not modify internal connector paths"
    );
}
```

This test proves:
1. Kerf changes the output (border is offset)
2. Subpath structure is preserved (same number of M commands)  
3. Internal connector paths are byte-identical with and without kerf
  </action>
  <verify>Run `cargo test -p puzzle-core test_kerf_only_offsets_border -- --nocapture` — test must pass.</verify>
  <done>Test proves kerf offsets border only and leaves connector geometry untouched.</done>
</task>

<task type="auto">
  <name>Task 3: Rebuild WASM and verify in browser</name>
  <files>web/pkg/</files>
  <action>
Rebuild the WASM package so the fix is available in the web GUI:

```bash
wasm-pack build crates/puzzle-wasm --target web --release --out-dir ../../web/pkg
```

After build, do a quick sanity check: the WASM binary should exist and be roughly the same size as before (~93KB gzipped range).
  </action>
  <verify>
  `ls -la web/pkg/puzzle_wasm_bg.wasm` exists and is a valid WASM file. Run `wasm-pack build` with no errors.
  </verify>
  <done>WASM package rebuilt with kerf fix; web GUI can be tested by opening `npm run dev` and adjusting kerf slider — geometry should no longer distort on internal edges.</done>
</task>

</tasks>

<verification>
1. `cargo test -p puzzle-core` — all tests pass (no regression + new kerf test)
2. `wasm-pack build crates/puzzle-wasm --target web --release --out-dir ../../web/pkg` succeeds
3. Visual: Open web GUI, set kerf to 0.2mm — border should be slightly larger, connector shapes should look identical to kerf=0
</verification>

<success_criteria>
- Kerf slider no longer distorts connector/tab geometry
- Border expands correctly by kerf_width/2 outward when kerf > 0
- All existing tests pass unchanged
- New test validates kerf isolation to border only
- WASM rebuilt and ready for browser testing
</success_criteria>

<output>
After completion, update .planning/STATE.md quick tasks table with task 008.
</output>
