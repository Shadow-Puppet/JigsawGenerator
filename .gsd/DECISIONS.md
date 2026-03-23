# Decisions Register

<!-- Append-only. Never edit or remove existing rows.
     To reverse a decision, add a new row that supersedes it.
     Read this file at the start of any planning or research phase. -->

| # | When | Scope | Decision | Choice | Rationale | Revisable? |
|---|------|-------|----------|--------|-----------|------------|
| D001 | M001 | arch | WASM boundary serialization | JSON serialization | Simple, debuggable, flexible | No |
| D002 | M001 | convention | Rust target setup | rustup locally for wasm32-unknown-unknown | Arch Linux system Rust lacks target | No |
| D003 | M001 | library | WASM loading in Vite | vite-plugin-wasm | Zero-config WASM loading | No |
| D004 | M001 | arch | Seed hashing | FNV-1a hash for string-to-u64 | Portable, deterministic, not std DefaultHasher | No |
| D005 | M001 | library | RNG crate config | rand with default-features=false | Avoids getrandom panic on wasm32 | No |
| D006 | M001 | arch | RNG ownership | RNG passed as &mut param to ConnectorGenerator | Grid controls determinism, not the connector | No |
| D007 | M001 | arch | Edge storage model | Shared-edge with index references into h_edges/v_edges | Pieces share edges; no duplication | No |
| D008 | M001 | convention | RNG consumption order | Fixed: all h_edges row-major then all v_edges row-major | Ensures seed determinism | No |
| D009 | M001 | arch | Connector RNG isolation | Separate RNG with seed suffix '-connectors' | Avoids disturbing grid construction RNG sequence | No |
| D010 | M001 | convention | Connector segment count | 5 cubic bezier segments per knob | baseline→neck, neck→body, top, body→neck, neck→baseline | No |
| D011 | M001 | convention | SVG structure | Single \<path\> with all cut lines | Border as closed subpath, internal edges as open subpaths | Yes — if whimsy requires separate paths |
| D012 | M001 | arch | Coordinate transforms | Affine (translate * rotate) for edge-local to global | kurbo::Affine, clean composition | No |
| D013 | M001 | convention | URL param format | Abbreviations (w/h, mm/in, integer percentages) | Compact shareable URLs | Yes — extend for whimsy params |
| D014 | M002 | arch | Geometric engine for mask/reverse-mask | linesweeper for boolean path ops on kurbo BezPaths | Pure Rust, kurbo-native, compiles to WASM, supports intersection/difference on bezier curves | Yes — if linesweeper proves unstable |
| D015 | M002 | arch | Core abstraction | Mask/reverse-mask — shape as stencil, caller picks which side to keep | Unifies custom borders (mask) and whimsy placement (reverse-mask) into one geometric operation | No |
| D016 | M002 | scope | Whimsy placement model | Free-form drag anywhere, no grid snap | User explicitly chose free-form over grid snap for natural feel | No |
| D017 | M002 | scope | Whimsy boundary connectors | No tabs on whimsy boundary — boundary itself is the cut line | Whimsy shape contour interlocks by shape, not by tabs; simplifies geometry significantly | No |
| D018 | M002 | scope | Whimsy count per puzzle | One whimsy at a time for v1 | Avoids whimsy-whimsy intersection complexity | Yes — when R014 is picked up |
| D019 | M002 | scope | Starter shape set | Heart + star | Proves system with both concave and convex shapes | Yes — extend library later |
| D020 | M002 | arch | Border mode grid behavior | Adaptive grid — piece count changes to fill the shape naturally | Better than clipping a fixed grid which leaves partial/empty cells | No |
