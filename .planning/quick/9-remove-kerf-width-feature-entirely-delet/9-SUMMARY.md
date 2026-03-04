---
phase: 9-remove-kerf
plan: 01
subsystem: puzzle-core, puzzle-wasm, web-frontend
tags: [cleanup, feature-removal, kerf]
dependency_graph:
  requires: []
  provides: [kerf-free-codebase]
  affects: [puzzle-core, puzzle-wasm, web-ui]
tech_stack:
  removed: [kerf offset algorithm, kerf UI controls, kerf URL parameter]
  patterns: [clean deletion across 3 layers]
key_files:
  deleted:
    - crates/puzzle-core/src/kerf.rs
  modified:
    - crates/puzzle-core/src/lib.rs
    - crates/puzzle-core/src/config.rs
    - crates/puzzle-core/src/svg_export.rs
    - crates/puzzle-core/src/grid.rs
    - crates/puzzle-wasm/src/lib.rs
    - web/src/main.ts
    - web/index.html
decisions: []
metrics:
  duration: 4 min
  completed: "2026-03-04T21:47:22Z"
  tasks_completed: 2
  tasks_total: 2
---

# Quick Task 9: Remove Kerf Width Feature Entirely

**One-liner:** Complete removal of broken kerf compensation feature across Rust core, WASM bridge, and web frontend

## What Was Done

### Task 1: Remove kerf from Rust core (puzzle-core) [`e614545`]
- **Deleted** `crates/puzzle-core/src/kerf.rs` — the entire offset_path algorithm and its 4 unit tests
- **Edited** `lib.rs` — removed `pub mod kerf;` and `pub use kerf::*;`
- **Edited** `config.rs` — removed `kerf_width: f64` field, its `#[serde(default)]` attribute, doc comment, Default impl line, validation check, `from_input()` parameter, and 3 kerf-specific tests (`test_validate_kerf_negative`, `test_validate_kerf_too_large`, `test_default_kerf_zero`). Updated all 5 `from_input()` test calls to remove the `0.0` kerf argument.
- **Edited** `svg_export.rs` — removed `use crate::kerf::offset_path;` import, the kerf offset block in `generate_svg()`, the doc comment line about kerf, `kerf_width: 0.0` from test helper, and the entire `test_kerf_only_offsets_border` test
- **Edited** `grid.rs` — removed `kerf_width: 0.0` from test helper
- **Verified:** `cargo test -p puzzle-core` — 97 tests pass, 0 failures

### Task 2: Remove kerf from WASM bridge and web frontend [`efb9242`]
- **Edited** `crates/puzzle-wasm/src/lib.rs` — removed kerf doc comments from `generate_svg`, removed `test_generate_svg_with_kerf` and `test_generate_svg_backward_compat` tests
- **Edited** `web/src/main.ts` — removed `kerfSlider`/`kerfReadout` variables, `kerf_width` from `buildConfig()`, `kerf` from URL read/write, kerf DOM cache lines, `kerfSlider` from sliders array, kerf readout update
- **Edited** `web/index.html` — removed the entire kerf slider `<div class="slider-group">` block (7 lines)
- **Verified:** `cargo test -p puzzle-wasm` — 13 tests pass, 0 failures
- **Rebuilt:** `wasm-pack build` — WASM package rebuilt successfully to `web/pkg/`
- **Verified:** `grep -ri kerf` across all source files returns zero matches

## Deviations from Plan

None - plan executed exactly as written.

## Verification Results

| Check | Result |
|-------|--------|
| `cargo test -p puzzle-core` | 97 passed, 0 failed |
| `cargo test -p puzzle-wasm` | 13 passed, 0 failed |
| `grep -ri kerf crates/ web/src/ web/index.html` | No matches (CLEAN) |
| `ls crates/puzzle-core/src/kerf.rs` | File does not exist |
| WASM rebuild | Success |

## Lines Removed

- **kerf.rs**: 280 lines (entire file)
- **config.rs**: ~35 lines (field, default, validation, tests, from_input param)
- **svg_export.rs**: ~55 lines (import, offset block, doc comment, test)
- **grid.rs**: 1 line (test helper field)
- **lib.rs** (puzzle-core): 2 lines (mod + re-export)
- **lib.rs** (puzzle-wasm): ~30 lines (doc comments + 2 tests)
- **main.ts**: ~15 lines (variables, config, URL, DOM, readout)
- **index.html**: 7 lines (slider block)
- **Total**: ~425 lines removed

## Self-Check: PASSED
