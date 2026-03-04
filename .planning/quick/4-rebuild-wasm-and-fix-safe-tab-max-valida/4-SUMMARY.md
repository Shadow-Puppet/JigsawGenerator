---
type: quick-summary
number: 4
description: "Rebuild WASM and fix safe_tab_max validation chicken-and-egg bug"
date: 2026-03-04
status: complete
commit: 2e1ce8e
---

# Quick Task 4: Summary

## What Changed

### crates/puzzle-wasm/src/lib.rs
- `safe_tab_max()` now clamps `size_pct` to [0.15, 0.25] and `taper` to [0.50, 1.20] before creating the grid
- This fixes the chicken-and-egg bug where an out-of-range slider value caused validation to fail, which prevented the max from ever being corrected

### web/src/main.ts
- `loadFromURL()` now clamps the URL-parsed `tab` value to [0.15, 0.25]
- Old URLs with `?tab=33` or `?tab=45` are now clamped to 25% on load

### WASM binary
- Rebuilt with `wasm-pack build --release` to include all quick-3 + quick-4 Rust changes
- The 25% cap in `safe_tab_max()` and `config.validate()` is now active at runtime

## Verification
- All 96 puzzle-core tests pass
- `npm run build` succeeds (web dist rebuilt)
- Tab slider range is now correctly 15%-25%
