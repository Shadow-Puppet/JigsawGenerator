# T02: 03-connector-generation-svg-export 02

**Slice:** S03 — **Milestone:** M001

## Description

Build the SVG export pipeline that walks the puzzle grid, transforms edge-local connector curves to global coordinates, constructs a single SVG path with border + internal edges + rounded corners, applies optional kerf compensation, and exposes it via a WASM endpoint.

Purpose: This transforms the in-memory puzzle geometry into a production-ready file users can send directly to their laser cutter. The SVG must be correct on first export — laser cutter software is unforgiving with malformed SVG.

Output: `svg_export.rs` module, `kerf.rs` module, `kerf_width` config field, `generate_svg` WASM endpoint, complete laser-cutter-ready SVG output.

## Must-Haves

- [ ] "Generated SVG contains a single path element with all cut lines, shared edges appearing exactly once"
- [ ] "SVG has explicit mm dimensions, matching viewBox, absolute coordinates, hairline black stroke"
- [ ] "Border edges are straight lines with quarter-circle rounded corners at the 4 puzzle corners"
- [ ] "Internal edges render as connector bezier curves transformed to global coordinates"
- [ ] "Kerf compensation offsets all paths outward by half the kerf width when kerf > 0"
- [ ] "WASM endpoint generate_svg() returns complete SVG string from PuzzleConfig JSON"

## Files

- `crates/puzzle-core/src/svg_export.rs`
- `crates/puzzle-core/src/kerf.rs`
- `crates/puzzle-core/src/config.rs`
- `crates/puzzle-core/src/lib.rs`
- `crates/puzzle-wasm/src/lib.rs`
