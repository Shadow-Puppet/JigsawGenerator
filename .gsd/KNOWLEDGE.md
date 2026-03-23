# Knowledge Base

<!-- Lessons learned, non-obvious patterns, and gotchas discovered during execution.
     Only add entries that would genuinely save future agents time. -->

## K001 — linesweeper requires closed paths (silent failure on open)

**Source:** M002/S01/T02
**Context:** Boolean ops via `linesweeper::binary_op` silently produce wrong results if input BezPaths are not closed (don't end with `ClosePath`). No error is returned — the output is just geometrically incorrect.
**Lesson:** Always verify shapes end with `close_path()` before passing to masking functions. The shape constructors enforce this, but any manually constructed paths must be checked.

## K002 — linesweeper Contours → BezPath conversion requires explicit PathEl iteration

**Source:** M002/S01/T02
**Context:** `linesweeper::binary_op` returns `topology::Contours`, not a `BezPath`. Each contour has a `.path` field that is a `BezPath`. To get a single usable path, you must iterate all contours and copy their `PathEl` elements into a new `BezPath`.
**Lesson:** The masking module handles this conversion. If downstream code needs per-contour access (e.g. to count separate regions), call `binary_op` directly rather than using the wrappers.

## K003 — BezPath translation requires manual PathEl iteration

**Source:** M002/S01/T02 (masking tests)
**Context:** kurbo `BezPath` has no built-in `translate(dx, dy)` method. To position a shape at a specific location, you must iterate all `PathEl` variants and offset each point manually. This is verbose (~15 lines per translation).
**Lesson:** S02 should add a `translate_path(path, dx, dy) -> BezPath` helper. The pattern from masking tests can be extracted into a utility function.

## K004 — Star path vertex count: N points = 2N vertices = 1 MoveTo + (2N-1) LineTo + 1 ClosePath

**Source:** M002/S01/T01
**Context:** A star with N outer points has N outer + N inner vertices = 2N total. In BezPath representation: 1 `MoveTo` for the first vertex, (2N-1) `LineTo` for the remaining vertices, and 1 `ClosePath`. The initial plan incorrectly stated "10 line segments" for a 5-pointed star — it's 9 `LineTo` segments.
**Lesson:** When counting path elements, remember `MoveTo` handles the first vertex. `LineTo` count = total_vertices - 1.

## K005 — linesweeper 0.3.0 compiles cleanly to wasm32-unknown-unknown

**Source:** M002/S01/T01
**Context:** The biggest risk for M002 was whether linesweeper (and its transitive deps) would compile to WASM. It does, cleanly, with no feature flag changes or workarounds needed.
**Lesson:** This risk is fully retired. Future slices can assume WASM compilation works for the boolean op stack.
