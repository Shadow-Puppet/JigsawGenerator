# 005 — Whimsy drag-drop & grid adaptation

**What:** User drags a heart/star shape onto the puzzle canvas, repositions and resizes it freely, and the grid adapts around the whimsy with its contour acting as the cut line (no tabs at the boundary).
**Why:** First user-facing reverse-mask feature. This proves the S02 `new_with_hole()` primitive end-to-end, including interactive responsiveness.

## Must-haves

- `whimsy_shape`, `whimsy_x`, `whimsy_y`, `whimsy_scale` fields on `PuzzleConfig` with `#[serde(default)]` (backward-compatible).
- `BoundaryPuzzle.hole: Option<BezPath>` field, populated by `new_with_hole()` and left `None` otherwise.
- `generate_boundary_svg()` and `boundary_border_to_binary()` emit the hole contour when `self.hole.is_some()` — both SVG and Canvas need it drawn.
- All three WASM endpoints handle the four combinations: neither / border only / whimsy only / both.
- `resolve_whimsy_shape(name, x, y, scale) -> BezPath` helper in the WASM layer, mirroring `resolve_border_shape` but applying translate + scale.
- Canvas drag-drop: click-to-place (once a shape is picked from the dropdown), drag to reposition, scroll-wheel-over-whimsy (or corner handles) to resize. Whimsy overlay redraws on every mousemove without calling WASM — stays fluid even if regeneration lags.
- Debounced WASM regeneration during drag (reuse `scheduleGenerate()` rAF throttle); immediate regeneration on mouseup.
- Whimsy state stored in puzzle mm coordinates — survives zoom/pan correctly.
- URL params `ws`, `wx`, `wy`, `wsc` persist whimsy across reloads.
- One whimsy at a time (R012). Selecting a new shape replaces the old.
- Remove-whimsy button clears state and regenerates.

## Task split (from original plan)

1. **Rust + WASM wiring** (~45 min). Config fields, `hole` field on `BoundaryPuzzle`, hole-contour export in SVG + binary, `resolve_whimsy_shape()`, endpoint branching for all four combinations, Rust tests.
2. **Frontend drag-drop UI** (~1h30m). HTML whimsy section (dropdown + remove button), drag/resize state machine, Canvas overlay in `drawPuzzle()`, `buildConfig()` wiring, URL params, debounced regeneration, piece-count display update.

## Verification

- `cargo test -p puzzle-core -- boundary` and `cargo test -p puzzle-wasm` both green.
- `cargo check --target wasm32-unknown-unknown -p puzzle-wasm` passes.
- `grep -q 'whimsy_shape' crates/puzzle-core/src/config.rs`, `grep -q 'resolve_whimsy_shape' crates/puzzle-wasm/src/lib.rs`, `grep -q 'whimsy' web/src/main.ts web/index.html`.
- Invalid whimsy shape name returns structured error JSON.
- Browser: drag heart onto puzzle → grid cells disappear under it, piece count decreases, heart outline renders as a cut line. Resize → grid re-adapts. Reload → whimsy restored. Download SVG → whimsy contour present.

## Integration points

- Consumes: `BoundaryPuzzle::new_with_hole()` (003), `heart_path`/`star_path` (002), `resolve_border_shape` pattern (003), `buildConfig`/`scheduleGenerate`/`drawPuzzle` (004).
- Produces for 006: whimsy placement state in the config + grid-with-hole, which the sub-puzzle slice clips a sub-grid against.
