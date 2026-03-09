---
phase: "17"
plan: 1
subsystem: puzzle-core, puzzle-wasm, web-ui
tags: [simplification, removal, border, ui-cleanup]
dependency_graph:
  requires: []
  provides:
    - "PuzzleConfig without BorderConfig or border field"
    - "Simple rectangular border path (no arcs)"
    - "UI without corner radius slider"
  affects:
    - crates/puzzle-core/src/config.rs
    - crates/puzzle-core/src/svg_export.rs
    - crates/puzzle-core/src/binary_export.rs
    - crates/puzzle-core/src/grid.rs
    - crates/puzzle-wasm/src/lib.rs
    - web/src/main.ts
    - web/index.html
tech_stack:
  patterns:
    - "Serde silently ignores unknown JSON fields for backward compat"
key_files:
  modified:
    - crates/puzzle-core/src/config.rs
    - crates/puzzle-core/src/svg_export.rs
    - crates/puzzle-core/src/binary_export.rs
    - crates/puzzle-core/src/grid.rs
    - crates/puzzle-wasm/src/lib.rs
    - web/src/main.ts
    - web/index.html
decisions:
  - "Sharp 90-degree corners as the only border style (no configurable radius)"
  - "Serde default behavior handles old URLs/configs that still include border field"
metrics:
  duration: "5 min"
  completed: "2026-03-09"
  tasks_completed: 2
  tasks_total: 2
---

# Quick Task 17: Remove Corner Radius Property Entirely

Eliminated BorderConfig struct, corner radius slider, and all rounded corner logic from the entire codebase for sharp 90-degree puzzle borders.

## Task Completion

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Remove BorderConfig from Rust core and simplify border path to rectangle | 8fb1020 | config.rs, svg_export.rs, binary_export.rs, grid.rs, lib.rs |
| 2 | Remove radius slider from UI and URL params, rebuild WASM | 4255ab8 | index.html, main.ts |

## Changes Made

### Rust Core (puzzle-core)
- **config.rs:** Deleted `BorderConfig` struct (3 impl blocks: struct, Default, validate). Removed `border` field from `PuzzleConfig` struct, `Default` impl, `validate()` method, and `from_input()` function. Removed 2 border validation tests and `BorderConfig` args from 6 test calls.
- **svg_export.rs:** Replaced 50-line `build_border_path()` with 7-line simple rectangle (`move_to(0,0)` -> 3 `line_to` -> `close_path`). Deleted `append_quarter_arc()` function (18 lines). Removed `Arc`, `Vec2`, `PI` imports (no longer needed). Updated doc comments.
- **binary_export.rs:** Removed `BorderConfig::default()` from test config helper.
- **grid.rs:** Removed `BorderConfig::default()` from test config helper.

### WASM Layer (puzzle-wasm)
- **lib.rs:** Removed `"border"` field from doc comment JSON example. Removed `,"border":{"corner_radius":2.0}` from all 10 test JSON strings.

### Web UI
- **index.html:** Deleted the corner radius slider group (7 lines: label, readout span, range input).
- **main.ts:** Removed `radiusSlider` and `radiusReadout` declarations. Removed `border` field from `buildConfig()`. Removed `radius` URL param from `loadFromURL()` and `updateURL()`. Removed `radiusReadout` update from `updateReadouts()`. Removed `radiusSlider` from slider event listener array.

## Verification Results
- `cargo test --workspace`: 118 tests pass (105 puzzle-core + 13 puzzle-wasm)
- `cargo clippy --workspace`: Only pre-existing warnings (unrelated to this change)
- `wasm-pack build --release`: Builds cleanly
- `npx tsc --noEmit`: No TypeScript errors
- Grep for `BorderConfig|corner_radius|radiusSlider|radiusReadout|radius-readout|append_quarter_arc` across crates/ and web/src/: **zero matches**
- Grep for `"radius"` in web/src/main.ts: **zero matches**

## Backward Compatibility
- Old URLs with `radius` param still load (JS `loadFromURL()` simply won't read it; unused params are harmless)
- Old JSON configs with `"border": {...}` still deserialize (serde silently ignores unknown fields by default)

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

- All 7 modified files exist on disk
- Commit 8fb1020 (Task 1) found in git log
- Commit 4255ab8 (Task 2) found in git log
