# M001: Migration

**Vision:** A procedural jigsaw puzzle pattern generator that outputs SVG cut paths for laser cutting.

## Success Criteria


## Slices

- [x] **S01: Build Pipeline Wasm Foundation** `risk:medium` `depends:[]`
  > After this: Set up the complete Rust-to-WASM-to-browser build pipeline with a working round-trip proof-of-life demo.
- [x] **S02: Grid Engine Data Model** `risk:medium` `depends:[S01]`
  > After this: Create all foundation types, configuration structs, seed module, edge types, and connector trait for the puzzle grid engine.
- [x] **S03: Connector Generation Svg Export** `risk:medium` `depends:[S02]`
  > After this: Implement the ClassicKnobConnector that produces traditional Ravensburger-style knob shapes using cubic bezier curves, and wire it into PuzzleGrid so all internal edges get connector geometry.
- [x] **S04: Web Gui Live Preview** `risk:medium` `depends:[S03]`
  > After this: Build the complete web GUI with parameter controls panel and live SVG preview.
