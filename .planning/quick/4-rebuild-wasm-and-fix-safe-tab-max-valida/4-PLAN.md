---
type: quick
number: 4
description: "Rebuild WASM and fix safe_tab_max validation chicken-and-egg bug"
date: 2026-03-04
---

# Quick Task 4: Rebuild WASM and fix safe_tab_max validation bug

## Problem

Quick task 3 changed Rust source to cap max tab size at 25%, but:
1. The WASM binary was NOT rebuilt — so the old 45% cap was still active at runtime
2. The `safe_tab_max()` WASM function validates config before computing max — if the current slider value is out of range (e.g. from a stale URL), validation fails, the error is silently caught, and the slider max never updates
3. `loadFromURL()` doesn't clamp tab values — old URLs with `?tab=33` bypass the HTML max attribute

## Tasks

### Task 1: Fix safe_tab_max and URL clamping
- **files:** `crates/puzzle-wasm/src/lib.rs`, `web/src/main.ts`
- **action:** Clamp tab params in `safe_tab_max()` before creating grid; clamp URL tab to [0.15, 0.25]
- **verify:** `cargo test` passes, `npm run build` succeeds
- **done:** Both fixes applied, WASM rebuilt, all 96 tests pass
