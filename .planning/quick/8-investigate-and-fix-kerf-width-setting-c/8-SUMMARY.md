---
phase: quick-008
plan: 1
subsystem: svg-export
tags: [kerf, border, svg, laser-cutting]
dependency_graph:
  requires: []
  provides: [kerf-border-only-offset]
  affects: [svg_export.rs, web-pkg]
tech_stack:
  added: []
  patterns: [split-path-construction, selective-kerf-offset]
key_files:
  created: []
  modified:
    - crates/puzzle-core/src/svg_export.rs
decisions:
  - Kerf offset applied to border path only; internal connector geometry preserved exactly
  - Split build_puzzle_path() into build_border_path() and build_connector_paths() for selective kerf application
  - Kept original build_puzzle_path() as backward-compat wrapper (marked #[allow(dead_code)])
metrics:
  duration: 3 min
  completed: "2026-03-04T19:59:42Z"
  tasks_completed: 3
  tasks_total: 3
---

# Quick Task 8: Fix Kerf Width Setting - Border Only Offset

Kerf compensation now offsets only the border path outward, leaving all internal connector/tab geometry unchanged for correct laser-cut piece fit.

## What Changed

### Task 1: Separate border and connector path construction, apply kerf to border only
**Commit:** `1b66b6c`

Refactored `svg_export.rs`:
- Split `build_puzzle_path()` into `build_border_path()` (closed border with rounded corners) and `build_connector_paths()` (open subpaths for internal edges)
- Updated `generate_svg()` to apply `offset_path()` to border only, then combine with unmodified connectors
- Added `use kurbo::PathEl` to imports for path element iteration
- Kept original `build_puzzle_path()` as thin wrapper for backward compatibility

**Root cause:** The old code called `offset_path(&path, kerf_width)` on the combined BezPath (border + all connectors). This distorted connector geometry because: (1) connectors are open subpaths where left-side normals don't have meaningful outward direction, (2) tab curves going in/out get uniformly shifted creating lopsided geometry, (3) internal edge kerf actually creates desirable clearance between pieces.

### Task 2: Add targeted test proving kerf only affects border dimensions
**Commit:** `b2d77de`

Added `test_kerf_only_offsets_border` test that:
1. Generates SVG with kerf=0.0 and kerf=0.2 using identical seed
2. Asserts SVGs differ (border is offset)
3. Asserts subpath count unchanged (same M command count)
4. Asserts connector paths after first Z are byte-identical

### Task 3: Rebuild WASM and verify in browser
**No commit** (web/pkg/ is gitignored — build artifact)

Successfully rebuilt WASM package with `wasm-pack build --target web --release`. Binary at `web/pkg/puzzle_wasm_bg.wasm` (170KB uncompressed, ~93KB gzipped range).

## Deviations from Plan

None - plan executed exactly as written.

## Test Results

```
running 105 tests
test result: ok. 105 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All 104 existing tests pass unchanged + 1 new kerf isolation test.

## Verification

- [x] `cargo test -p puzzle-core` — 105 tests pass
- [x] `wasm-pack build crates/puzzle-wasm --target web --release` — succeeds
- [x] Kerf=0 produces identical SVG to before (no regression)
- [x] Kerf>0 offsets only the border, not internal connector edges
- [x] Internal connector paths byte-identical regardless of kerf value

## Self-Check: PASSED
