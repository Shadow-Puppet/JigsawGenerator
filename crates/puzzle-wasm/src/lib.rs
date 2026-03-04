use serde::Serialize;
use wasm_bindgen::prelude::*;

use puzzle_core::{
    compute_piece_breakdown, ClassicKnobConnector, GridConfig, PieceType, PuzzleConfig, PuzzleGrid,
};

/// Initialize the panic hook for better error messages in the browser console.
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Compute puzzle piece breakdown from a JSON configuration string.
///
/// Expects JSON: `{"rows": N, "cols": M}`
/// Returns JSON: `{"total": N, "corners": N, "edges": N, "interior": N}`
/// Or on error: `{"error": "message"}`
#[wasm_bindgen]
pub fn compute_pieces(config_json: &str) -> String {
    let config: GridConfig = match serde_json::from_str(config_json) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"Invalid JSON: {}"}}"#, e),
    };

    match compute_piece_breakdown(&config) {
        Ok(breakdown) => serde_json::to_string(&breakdown)
            .unwrap_or_else(|e| format!(r#"{{"error":"Serialization error: {}"}}"#, e)),
        Err(msg) => format!(r#"{{"error":"{}"}}"#, msg),
    }
}

// ─── WASM Response Types ──────────────────────────────────────────

/// Simplified piece info for JSON output (no internal edge indices).
#[derive(Debug, Serialize)]
struct PieceInfo {
    row: usize,
    col: usize,
    piece_type: String,
    is_top_border: bool,
    is_bottom_border: bool,
    is_left_border: bool,
    is_right_border: bool,
}

/// Piece count breakdown in the grid response.
#[derive(Debug, Serialize)]
struct PieceBreakdownInfo {
    total: usize,
    corners: usize,
    edges: usize,
    interior: usize,
}

/// Edge count summary in the grid response.
#[derive(Debug, Serialize)]
struct EdgeSummary {
    h_edge_count: usize,
    v_edge_count: usize,
    border_count: usize,
    internal_count: usize,
}

/// Full grid response returned by the generate_grid WASM endpoint.
///
/// This is a WASM-layer concern — it selects what data to expose to JS,
/// not the full internal Edge/Piece structs. Full edge geometry (bezier
/// control points) will be added in Phase 3 when connectors are generated.
#[derive(Debug, Serialize)]
struct GridResponse {
    rows: u32,
    cols: u32,
    width_mm: f64,
    height_mm: f64,
    seed: String,
    piece_breakdown: PieceBreakdownInfo,
    edge_summary: EdgeSummary,
    pieces: Vec<PieceInfo>,
}

/// Generate a puzzle grid from a JSON configuration string.
///
/// Accepts full PuzzleConfig JSON:
/// ```json
/// {
///   "rows": 6, "cols": 8,
///   "width": 297.0, "height": 210.0,
///   "unit": "Millimeters",
///   "tab": { "size_pct": 0.25, "taper": 0.5 },
///   "border": { "corner_radius": 2.0 },
///   "seed": "my-puzzle-seed"
/// }
/// ```
///
/// Returns JSON GridResponse with grid summary, piece breakdown,
/// edge summary, and per-piece info.
///
/// On error: `{"error": "message"}`
///
/// **Seed handling:** If `seed` is empty, a fixed default seed "default" is used.
/// WASM cannot access OS entropy (no getrandom), so true random seeds must be
/// generated in JavaScript and passed in. Phase 4 will pass a JS-generated
/// random seed from the browser.
#[wasm_bindgen]
pub fn generate_grid(config_json: &str) -> String {
    // 1. Deserialize PuzzleConfig from JSON
    let mut config: PuzzleConfig = match serde_json::from_str(config_json) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"Invalid JSON: {}"}}"#, e),
    };

    // Handle empty seed: use fixed default since WASM has no OS entropy.
    // Phase 4 will pass JS-generated random seeds from the browser.
    if config.seed.is_empty() {
        config.seed = "default".to_string();
    }

    let seed_used = config.seed.clone();

    // 2. Create PuzzleGrid (validates config internally)
    let grid = match PuzzleGrid::new(config) {
        Ok(g) => g,
        Err(e) => return format!(r#"{{"error":"{}"}}"#, e),
    };

    // 3. Build response
    let rows = grid.config.rows;
    let cols = grid.config.cols;
    let rows_usize = rows as usize;
    let cols_usize = cols as usize;

    // Piece breakdown
    let all_pieces = grid.pieces();
    let corners = all_pieces
        .iter()
        .filter(|p| p.piece_type == PieceType::Corner)
        .count();
    let edges = all_pieces
        .iter()
        .filter(|p| p.piece_type == PieceType::Edge)
        .count();
    let interior = all_pieces
        .iter()
        .filter(|p| p.piece_type == PieceType::Interior)
        .count();

    // Edge summary
    let border_count = grid
        .h_edges
        .iter()
        .chain(grid.v_edges.iter())
        .filter(|e| e.is_border)
        .count();
    let total_edges = grid.h_edges.len() + grid.v_edges.len();
    let internal_count = total_edges - border_count;

    // Per-piece info
    let pieces: Vec<PieceInfo> = all_pieces
        .iter()
        .map(|p| {
            let piece_type_str = match p.piece_type {
                PieceType::Corner => "corner",
                PieceType::Edge => "edge",
                PieceType::Interior => "interior",
            };
            PieceInfo {
                row: p.row,
                col: p.col,
                piece_type: piece_type_str.to_string(),
                is_top_border: p.row == 0,
                is_bottom_border: p.row == rows_usize - 1,
                is_left_border: p.col == 0,
                is_right_border: p.col == cols_usize - 1,
            }
        })
        .collect();

    let response = GridResponse {
        rows,
        cols,
        width_mm: grid.config.width,
        height_mm: grid.config.height,
        seed: seed_used,
        piece_breakdown: PieceBreakdownInfo {
            total: all_pieces.len(),
            corners,
            edges,
            interior,
        },
        edge_summary: EdgeSummary {
            h_edge_count: grid.h_edges.len(),
            v_edge_count: grid.v_edges.len(),
            border_count,
            internal_count,
        },
        pieces,
    };

    // 4. Serialize to JSON
    serde_json::to_string(&response)
        .unwrap_or_else(|e| format!(r#"{{"error":"Serialization error: {}"}}"#, e))
}

/// Generate a laser-cutter-ready SVG from a JSON configuration string.
///
/// Accepts full PuzzleConfig JSON (same as `generate_grid`).
///
/// Returns a complete SVG string with:
/// - Physical mm dimensions and viewBox
/// - Single `<path>` with all cut lines (border + connectors)
/// - Hairline black stroke, no fill
///
/// On error: `{"error": "message"}`
#[wasm_bindgen]
pub fn generate_svg(config_json: &str) -> String {
    // 1. Deserialize PuzzleConfig from JSON
    let mut config: PuzzleConfig = match serde_json::from_str(config_json) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"Invalid JSON: {}"}}"#, e),
    };

    // Handle empty seed: use fixed default since WASM has no OS entropy.
    if config.seed.is_empty() {
        config.seed = "default".to_string();
    }

    // 2. Create PuzzleGrid (validates config internally)
    let mut grid = match PuzzleGrid::new(config) {
        Ok(g) => g,
        Err(e) => return format!(r#"{{"error":"{}"}}"#, e),
    };

    // 3. Generate connectors on all internal edges
    grid.generate_connectors(&ClassicKnobConnector);

    // 4. Generate and return SVG
    puzzle_core::generate_svg(&grid)
}

/// Compute the safe maximum tab size for a given grid configuration.
///
/// Accepts JSON: `{"rows": N, "cols": M, "width": W, "height": H}`
/// Returns JSON: `{"max": 0.25}` (the safe maximum tab size_pct)
/// Or on error: `{"error": "message"}`
///
/// Note: Clamps tab/taper to valid ranges before creating the grid so this
/// function works even when the current slider value is out of range (e.g.
/// during initial load from a stale URL).
#[wasm_bindgen]
pub fn safe_tab_max(config_json: &str) -> String {
    let mut config: PuzzleConfig = match serde_json::from_str(config_json) {
        Ok(c) => c,
        Err(e) => return format!(r#"{{"error":"Invalid JSON: {}"}}"#, e),
    };

    // Clamp tab params to valid ranges so PuzzleGrid::new() doesn't reject
    // the config — the whole point of this function is to *compute* the max.
    config.tab.size_pct = config.tab.size_pct.clamp(0.15, 0.25);
    config.tab.taper = config.tab.taper.clamp(0.50, 1.20);
    if let Some(ref mut max) = config.tab.size_pct_max {
        *max = max.clamp(0.15, 0.25);
    }
    if let Some(ref mut max) = config.tab.taper_max {
        *max = max.clamp(0.50, 1.20);
    }

    let grid = match PuzzleGrid::new(config) {
        Ok(g) => g,
        Err(e) => return format!(r#"{{"error":"{}"}}"#, e),
    };

    let max = grid.safe_tab_max();
    format!(r#"{{"max":{:.4}}}"#, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_grid_valid_config() {
        let config_json = r#"{"rows":6,"cols":8,"width":297.0,"height":210.0,"unit":"Millimeters","tab":{"size_pct":0.25},"border":{"corner_radius":2.0},"seed":"test-seed"}"#;
        let result = generate_grid(config_json);

        // Should be valid JSON, not an error
        assert!(!result.contains(r#""error""#), "Got error: {}", result);

        // Parse and check fields
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["rows"], 6);
        assert_eq!(parsed["cols"], 8);
        assert_eq!(parsed["width_mm"], 297.0);
        assert_eq!(parsed["height_mm"], 210.0);
        assert_eq!(parsed["seed"], "test-seed");

        // Check piece breakdown
        assert_eq!(parsed["piece_breakdown"]["total"], 48);
        assert_eq!(parsed["piece_breakdown"]["corners"], 4);

        // Check edge summary exists
        assert!(parsed["edge_summary"]["h_edge_count"].is_number());
        assert!(parsed["edge_summary"]["v_edge_count"].is_number());

        // Check pieces array
        let pieces = parsed["pieces"].as_array().unwrap();
        assert_eq!(pieces.len(), 48);
    }

    #[test]
    fn test_generate_grid_deterministic() {
        let config_json = r#"{"rows":4,"cols":5,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"border":{"corner_radius":2.0},"seed":"determinism-test"}"#;
        let result1 = generate_grid(config_json);
        let result2 = generate_grid(config_json);

        assert_eq!(result1, result2, "Same seed must produce identical output");
    }

    #[test]
    fn test_generate_grid_empty_seed_uses_default() {
        let config_json = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"border":{"corner_radius":2.0},"seed":""}"#;
        let result = generate_grid(config_json);
        assert!(!result.contains(r#""error""#), "Got error: {}", result);

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["seed"], "default");
    }

    #[test]
    fn test_generate_grid_invalid_json() {
        let result = generate_grid("not valid json");
        assert!(result.contains(r#""error""#));
    }

    #[test]
    fn test_generate_grid_invalid_config() {
        // rows=1 is below minimum (2)
        let config_json = r#"{"rows":1,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"border":{"corner_radius":2.0},"seed":"test"}"#;
        let result = generate_grid(config_json);
        assert!(result.contains(r#""error""#));
    }

    #[test]
    fn test_generate_grid_piece_types_correct() {
        let config_json = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"border":{"corner_radius":2.0},"seed":"piece-types"}"#;
        let result = generate_grid(config_json);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let pieces = parsed["pieces"].as_array().unwrap();

        // 3x4 grid: 4 corners, 6 edges, 2 interior
        let corners = pieces
            .iter()
            .filter(|p| p["piece_type"] == "corner")
            .count();
        let edges = pieces.iter().filter(|p| p["piece_type"] == "edge").count();
        let interior = pieces
            .iter()
            .filter(|p| p["piece_type"] == "interior")
            .count();

        assert_eq!(corners, 4);
        assert_eq!(edges, 6);
        assert_eq!(interior, 2);
    }

    #[test]
    fn test_generate_grid_edge_counts() {
        let config_json = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"border":{"corner_radius":2.0},"seed":"edge-counts"}"#;
        let result = generate_grid(config_json);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // 3x4 grid: h_edges = (3+1)*4 = 16, v_edges = 3*(4+1) = 15
        assert_eq!(parsed["edge_summary"]["h_edge_count"], 16);
        assert_eq!(parsed["edge_summary"]["v_edge_count"], 15);
    }

    #[test]
    fn test_compute_pieces_still_works() {
        // Backward compatibility: compute_pieces endpoint unchanged
        let result = compute_pieces(r#"{"rows":3,"cols":4}"#);
        assert!(!result.contains(r#""error""#));

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["total"], 12);
        assert_eq!(parsed["corners"], 4);
        assert_eq!(parsed["edges"], 6);
        assert_eq!(parsed["interior"], 2);
    }

    #[test]
    fn test_generate_grid_json_roundtrip() {
        let config_json = r#"{"rows":6,"cols":8,"width":297.0,"height":210.0,"unit":"Millimeters","tab":{"size_pct":0.25},"border":{"corner_radius":2.0},"seed":"test-seed"}"#;
        let result = generate_grid(config_json);

        // Verify it's valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&result);
        assert!(parsed.is_ok(), "Result is not valid JSON: {}", result);

        // Verify no error
        let value = parsed.unwrap();
        assert!(value.get("error").is_none(), "Got error: {}", result);

        // Verify expected fields exist
        assert!(value.get("rows").is_some());
        assert!(value.get("cols").is_some());
        assert!(value.get("width_mm").is_some());
        assert!(value.get("height_mm").is_some());
        assert!(value.get("seed").is_some());
        assert!(value.get("piece_breakdown").is_some());
        assert!(value.get("edge_summary").is_some());
        assert!(value.get("pieces").is_some());
    }

    // ─── generate_svg Tests ───────────────────────────────────────

    #[test]
    fn test_generate_svg_returns_svg() {
        let config_json = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"border":{"corner_radius":2.0},"seed":"svg-test"}"#;
        let result = generate_svg(config_json);
        assert!(
            result.starts_with("<svg"),
            "should start with <svg, got: {}...",
            &result[..50.min(result.len())]
        );
        assert!(result.contains("</svg>"), "should contain </svg>");
        assert!(!result.contains(r#""error""#), "should not be error JSON");
    }

    #[test]
    fn test_generate_svg_has_connectors() {
        let config_json = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"border":{"corner_radius":2.0},"seed":"conn-svg"}"#;
        let result = generate_svg(config_json);
        // Extract path data
        let d_start = result.find("d='").expect("should have d attribute") + 3;
        let d_end = result[d_start..].find('\'').unwrap() + d_start;
        let path_data = &result[d_start..d_end];
        assert!(
            path_data.contains('C'),
            "path data should contain C commands (cubic bezier curves from connectors)"
        );
    }

    #[test]
    fn test_generate_svg_deterministic() {
        let config_json = r#"{"rows":3,"cols":4,"width":200.0,"height":150.0,"unit":"Millimeters","tab":{"size_pct":0.25},"border":{"corner_radius":2.0},"seed":"determ-svg"}"#;
        let svg1 = generate_svg(config_json);
        let svg2 = generate_svg(config_json);
        assert_eq!(svg1, svg2, "same config must produce identical SVG");
    }

    #[test]
    fn test_generate_svg_invalid_json() {
        let result = generate_svg("not valid json");
        assert!(
            result.contains(r#""error""#),
            "should return error JSON for invalid input"
        );
    }
}
