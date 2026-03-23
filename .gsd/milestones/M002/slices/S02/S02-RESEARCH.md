# S02: Boundary-Aware Grid Generation — Research

**Date:** 2026-03-21
**Depth:** Deep research — high risk, novel architecture, multiple interacting subsystems

## Summary

S02 must make `PuzzleGrid` work inside non-rectangular boundaries (R002) and with holes cut out for whimsy shapes (R004), while preserving seed determinism (R013). The approach is a post-processing pipeline on top of the existing rectangular grid: generate the full rectangular grid first (preserving RNG sequence for determinism), then classify cells as inside/outside the boundary, remove outside cells, and replace boundary-adjacent grid edges with the shape's contour segments.

The critical architectural insight is that this should NOT modify `PuzzleGrid` internals. Instead, a new `BoundaryPuzzle` struct wraps a standard `PuzzleGrid` and adds boundary awareness. The existing `generate_connectors()` runs first on the full grid, then `BoundaryPuzzle` filters which edges to include in export. This preserves the existing RNG sequence exactly.

## Key Findings

- kurbo 0.13 `BezPath` implements `Shape` trait with `contains(point)` — cell classification is straightforward
- Binary export border channel already uses variable-length command format — shape contour maps directly
- Shape paths center within (width, height) matching puzzle coordinate space — no translation needed
- All 114 existing tests pass; boundary logic is fully additive

## Architecture

New `boundary.rs` with `BoundaryPuzzle` wrapper. Three tasks: core engine, SVG export, WASM integration.