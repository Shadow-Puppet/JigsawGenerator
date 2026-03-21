# T01: 03-connector-generation-svg-export 01

**Slice:** S03 — **Milestone:** M001

## Description

Implement the ClassicKnobConnector that produces traditional Ravensburger-style knob shapes using cubic bezier curves, and wire it into PuzzleGrid so all internal edges get connector geometry.

Purpose: This is the core geometric algorithm that transforms a bare grid into a real jigsaw puzzle. Without connectors, pieces are just rectangles. The connector shapes determine whether laser-cut pieces interlock properly.

Output: `ClassicKnobConnector` struct implementing `ConnectorGenerator`, `PuzzleGrid::generate_connectors()` method, all internal edges populated with bezier curves.

## Must-Haves

- [ ] "ClassicKnobConnector produces cubic bezier curves forming a traditional knob shape"
- [ ] "Each edge has procedural variation from jitter (different control point positions, different knob center offset)"
- [ ] "TabDirection::Out produces knob in +Y direction, TabDirection::In produces knob in -Y direction"
- [ ] "Same seed produces identical connector curves; different seeds produce different curves"
- [ ] "PuzzleGrid.generate_connectors() populates all internal edge connector fields"

## Files

- `crates/puzzle-core/src/classic_connector.rs`
- `crates/puzzle-core/src/grid.rs`
- `crates/puzzle-core/src/lib.rs`
