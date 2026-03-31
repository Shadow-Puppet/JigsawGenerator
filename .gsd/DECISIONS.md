# Decisions Register

<!-- Append-only. Never edit or remove existing rows.
     To reverse a decision, add a new row that supersedes it.
     Read this file at the start of any planning or research phase. -->

| # | When | Scope | Decision | Choice | Rationale | Revisable? | Made By |
|---|------|-------|----------|--------|-----------|------------|---------|
| D001 | M001 | arch | WASM boundary serialization | JSON serialization | Simple, debuggable, flexible | No | agent |
| D002 | M001 | convention | Rust target setup | rustup locally for wasm32-unknown-unknown | Arch Linux system Rust lacks target | No | agent |
| D003 | M001 | library | WASM loading in Vite | vite-plugin-wasm | Zero-config WASM loading | No | agent |
| D004 | M001 | arch | Seed hashing | FNV-1a hash for string-to-u64 | Portable, deterministic, not std DefaultHasher | No | agent |
| D005 | M001 | library | RNG crate config | rand with default-features=false | Avoids getrandom panic on wasm32 | No | agent |
| D006 | M001 | arch | RNG ownership | RNG passed as &mut param to ConnectorGenerator | Grid controls determinism, not the connector | No | agent |
| D007 | M001 | arch | Edge storage model | Shared-edge with index references into h_edges/v_edges | Pieces share edges; no duplication | No | agent |
| D008 | M001 | convention | RNG consumption order | Fixed: all h_edges row-major then all v_edges row-major | Ensures seed determinism | No | agent |
| D009 | M001 | arch | Connector RNG isolation | Separate RNG with seed suffix '-connectors' | Avoids disturbing grid construction RNG sequence | No | agent |
| D010 | M001 | convention | Connector segment count | 5 cubic bezier segments per knob | baseline→neck, neck→body, top, body→neck, neck→baseline | No | agent |
| D011 | M001 | convention | SVG structure | Single \<path\> with all cut lines | Border as closed subpath, internal edges as open subpaths | Yes — if whimsy requires separate paths | agent |
| D012 | M001 | arch | Coordinate transforms | Affine (translate * rotate) for edge-local to global | kurbo::Affine, clean composition | No | agent |
| D013 | M001 | convention | URL param format | Abbreviations (w/h, mm/in, integer percentages) | Compact shareable URLs | Yes — extend for whimsy params | agent |
| D014 | M002 | arch | Geometric engine for mask/reverse-mask | linesweeper for boolean path ops on kurbo BezPaths | Pure Rust, kurbo-native, compiles to WASM, supports intersection/difference on bezier curves | Yes — if linesweeper proves unstable | agent |
| D015 | M002 | arch | Core abstraction | Mask/reverse-mask — shape as stencil, caller picks which side to keep | Unifies custom borders (mask) and whimsy placement (reverse-mask) into one geometric operation | No | agent |
| D016 | M002 | scope | Whimsy placement model | Free-form drag anywhere, no grid snap | User explicitly chose free-form over grid snap for natural feel | No | agent |
| D017 | M002 | scope | Whimsy boundary connectors | No tabs on whimsy boundary — boundary itself is the cut line | Whimsy shape contour interlocks by shape, not by tabs; simplifies geometry significantly | No | agent |
| D018 | M002 | scope | Whimsy count per puzzle | One whimsy at a time for v1 | Avoids whimsy-whimsy intersection complexity | Yes — when R014 is picked up | agent |
| D019 | M002 | scope | Starter shape set | Heart + star | Proves system with both concave and convex shapes | Yes — extend library later | agent |
| D020 | M002 | arch | Border mode grid behavior | Adaptive grid — piece count changes to fill the shape naturally | Better than clipping a fixed grid which leaves partial/empty cells | No | agent |
| D021 | M002/S01 | convention | Star inner radius ratio | 40% of outer radius | Per plan specification; produces a classic 5-pointed star look with distinct points and visible indentations | Yes | agent |
| D022 | M002/S02/T03 | library | Whether to add kurbo as direct dependency to puzzle-wasm or avoid naming BezPath type | Added kurbo = "0.13" as direct dependency to puzzle-wasm Cargo.toml | The resolve_border_shape() helper needs to return a kurbo::BezPath. Since kurbo is already transitively included via puzzle-core, adding it as a direct dependency adds zero binary bloat and enables clean type signatures in the WASM crate. | Yes | agent |
| D023 | M002/S02/T01 | arch | BoundaryPuzzle cell classification data structure | Vec<Vec<bool>> grid for O(1) cell inclusion lookup; edge accessors return indices into grid.h_edges/v_edges (not copies) | O(1) lookup needed during edge filtering (iterating all edges). Returning indices avoids copying edge data and lets downstream code access connectors directly from the grid arrays. | Yes | agent |
| D024 | M002/S02/T01 | arch | Boundary filtering strategy for determinism preservation | BoundaryPuzzle wraps a full rectangular PuzzleGrid and filters post-generation — RNG sequence is identical regardless of boundary shape | Generating the full grid first then filtering ensures the RNG produces the same sequence for a given seed, regardless of which cells end up inside the boundary. If we skipped cells during generation, the RNG sequence would diverge depending on the shape. | Yes | agent |
| D025 | M002/S02/T03 | arch | Shape name → BezPath resolution centralization in WASM layer | resolve_border_shape(name, width, height) → BezPath as single mapping function; new shapes require only one match arm addition | All three WASM endpoints need to convert the user-facing shape name string to a scaled BezPath. Centralizing this avoids duplication and ensures consistent behavior (error messages, scaling) across endpoints. | Yes | agent |
| D026 | M002/S03/T02 | M002/S03 | Border shape dropdown default value representation | Empty string value for Rectangle (default) option; buildConfig() omits border_shape entirely when empty | Avoids sending 'rectangle' to Rust which would trigger 'Unknown border shape' error from serde. Omitting the field preserves backward compatibility — existing configs without border_shape produce rectangular puzzles as before. | Yes | agent |
| D027 | M002/S03/T02 | M002/S03 | Piece count display format for non-rectangular puzzles | Boundary puzzles show 'N pieces (shape border)' instead of corner/edge/interior breakdown | Corner/edge/interior classification assumes rectangular geometry and is misleading for boundary puzzles where the grid is clipped. The simplified format still shows the accurate WASM-returned piece count and indicates the border is active. | Yes — could add boundary-aware piece classification later | agent |
