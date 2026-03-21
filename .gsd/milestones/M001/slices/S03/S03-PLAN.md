# S03: Connector Generation Svg Export

**Goal:** Implement the ClassicKnobConnector that produces traditional Ravensburger-style knob shapes using cubic bezier curves, and wire it into PuzzleGrid so all internal edges get connector geometry.
**Demo:** Implement the ClassicKnobConnector that produces traditional Ravensburger-style knob shapes using cubic bezier curves, and wire it into PuzzleGrid so all internal edges get connector geometry.

## Must-Haves


## Tasks

- [x] **T01: 03-connector-generation-svg-export 01** `est:5min`
  - Implement the ClassicKnobConnector that produces traditional Ravensburger-style knob shapes using cubic bezier curves, and wire it into PuzzleGrid so all internal edges get connector geometry.

Purpose: This is the core geometric algorithm that transforms a bare grid into a real jigsaw puzzle. Without connectors, pieces are just rectangles. The connector shapes determine whether laser-cut pieces interlock properly.

Output: `ClassicKnobConnector` struct implementing `ConnectorGenerator`, `PuzzleGrid::generate_connectors()` method, all internal edges populated with bezier curves.
- [x] **T02: 03-connector-generation-svg-export 02** `est:6min`
  - Build the SVG export pipeline that walks the puzzle grid, transforms edge-local connector curves to global coordinates, constructs a single SVG path with border + internal edges + rounded corners, applies optional kerf compensation, and exposes it via a WASM endpoint.

Purpose: This transforms the in-memory puzzle geometry into a production-ready file users can send directly to their laser cutter. The SVG must be correct on first export — laser cutter software is unforgiving with malformed SVG.

Output: `svg_export.rs` module, `kerf.rs` module, `kerf_width` config field, `generate_svg` WASM endpoint, complete laser-cutter-ready SVG output.

## Files Likely Touched

- `crates/puzzle-core/src/classic_connector.rs`
- `crates/puzzle-core/src/grid.rs`
- `crates/puzzle-core/src/lib.rs`
- `crates/puzzle-core/src/svg_export.rs`
- `crates/puzzle-core/src/kerf.rs`
- `crates/puzzle-core/src/config.rs`
- `crates/puzzle-core/src/lib.rs`
- `crates/puzzle-wasm/src/lib.rs`
