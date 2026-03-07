use std::f64::consts::PI;

use kurbo::{Affine, Arc, BezPath, PathEl, Point, Vec2};

use crate::grid::PuzzleGrid;

/// Generate a complete SVG document from a populated PuzzleGrid.
///
/// The grid must have had `generate_connectors()` called so that
/// internal edges have bezier connector curves.
///
/// Returns a complete SVG string with:
/// - Physical mm dimensions and matching viewBox
/// - A single `<path>` element with all cut lines
/// - Hairline black stroke, no fill
/// - Absolute coordinates only (M, L, C, Z)
/// - Border as a closed subpath with rounded corners
/// - Internal edges as open subpaths with connector curves
pub fn generate_svg(grid: &PuzzleGrid) -> String {
    let border = build_border_path(grid);
    let connectors = build_connector_paths(grid);

    // Combine: border first, then connectors
    let mut combined = border;
    for el in connectors.iter() {
        match el {
            PathEl::MoveTo(p) => combined.move_to(p),
            PathEl::LineTo(p) => combined.line_to(p),
            PathEl::CurveTo(p1, p2, p3) => combined.curve_to(p1, p2, p3),
            PathEl::ClosePath => combined.close_path(),
            _ => {}
        }
    }

    let path_data = combined.to_svg();
    build_svg_document(&path_data, grid.config.width, grid.config.height)
}

/// Construct the puzzle cut path as a single BezPath.
///
/// The path contains:
/// 1. A closed border subpath with straight edges and quarter-circle
///    rounded corners at the 4 puzzle corners
/// 2. Open subpaths for each internal edge, with connector bezier curves
///    transformed from edge-local to global coordinates
#[allow(dead_code)]
fn build_puzzle_path(grid: &PuzzleGrid) -> BezPath {
    let border = build_border_path(grid);
    let connectors = build_connector_paths(grid);

    let mut path = border;
    for el in connectors.iter() {
        match el {
            PathEl::MoveTo(p) => path.move_to(p),
            PathEl::LineTo(p) => path.line_to(p),
            PathEl::CurveTo(p1, p2, p3) => path.curve_to(p1, p2, p3),
            PathEl::ClosePath => path.close_path(),
            _ => {}
        }
    }
    path
}

/// Construct the border as a closed BezPath with rounded corners.
///
/// Walk clockwise: top → right → bottom → left with quarter-circle arcs
/// at each corner. Returns a single closed subpath.
pub(crate) fn build_border_path(grid: &PuzzleGrid) -> BezPath {
    let mut path = BezPath::new();

    let w = grid.config.width;
    let h = grid.config.height;
    let r = grid.config.border.corner_radius;

    // Clamp radius to avoid exceeding half the smallest dimension
    let r = r.min(w / 2.0).min(h / 2.0);

    // Start at top edge, after top-left corner
    path.move_to(Point::new(r, 0.0));

    // Top edge: straight line to before top-right corner
    path.line_to(Point::new(w - r, 0.0));

    // Top-right corner arc (quarter circle, clockwise)
    if r > 0.0 {
        append_quarter_arc(&mut path, Point::new(w - r, r), r, -PI / 2.0, PI / 2.0);
    }

    // Right edge: straight line to before bottom-right corner
    path.line_to(Point::new(w, h - r));

    // Bottom-right corner arc
    if r > 0.0 {
        append_quarter_arc(&mut path, Point::new(w - r, h - r), r, 0.0, PI / 2.0);
    }

    // Bottom edge: straight line to before bottom-left corner
    path.line_to(Point::new(r, h));

    // Bottom-left corner arc
    if r > 0.0 {
        append_quarter_arc(&mut path, Point::new(r, h - r), r, PI / 2.0, PI / 2.0);
    }

    // Left edge: straight line to before top-left corner
    path.line_to(Point::new(0.0, r));

    // Top-left corner arc
    if r > 0.0 {
        append_quarter_arc(&mut path, Point::new(r, r), r, PI, PI / 2.0);
    }

    // Close the border
    path.close_path();

    path
}

/// Construct all internal edge connector curves as open subpaths.
///
/// Each internal edge becomes an open subpath starting with MoveTo at
/// the first control point, followed by CurveTo segments for the
/// connector bezier curves.
fn build_connector_paths(grid: &PuzzleGrid) -> BezPath {
    let mut path = BezPath::new();

    let rows = grid.config.rows as usize;
    let cols = grid.config.cols as usize;

    // Internal horizontal edges: rows 1..rows (skip row 0 = top border, row rows = bottom border)
    for row in 1..rows {
        for col in 0..cols {
            let edge = grid.h_edge(row, col);
            if edge.is_border {
                continue;
            }
            if let Some(ref curves) = edge.connector {
                let transform = edge_transform(edge.start, edge.end);
                let first_p0 = transform * curves[0].p0;
                path.move_to(first_p0);
                for curve in curves {
                    let p1 = transform * curve.p1;
                    let p2 = transform * curve.p2;
                    let p3 = transform * curve.p3;
                    path.curve_to(p1, p2, p3);
                }
            }
        }
    }

    // Internal vertical edges: cols 1..cols (skip col 0 = left border, col cols = right border)
    for row in 0..rows {
        for col in 1..cols {
            let edge = grid.v_edge(row, col);
            if edge.is_border {
                continue;
            }
            if let Some(ref curves) = edge.connector {
                let transform = edge_transform(edge.start, edge.end);
                let first_p0 = transform * curves[0].p0;
                path.move_to(first_p0);
                for curve in curves {
                    let p1 = transform * curve.p1;
                    let p2 = transform * curve.p2;
                    let p3 = transform * curve.p3;
                    path.curve_to(p1, p2, p3);
                }
            }
        }
    }

    path
}

/// Compute the affine transform from edge-local to global coordinates.
///
/// Edge-local: origin at edge start, x-axis along edge direction.
/// Global: the puzzle coordinate system in mm.
pub(crate) fn edge_transform(start: Point, end: Point) -> Affine {
    let angle = (end.y - start.y).atan2(end.x - start.x);
    Affine::translate(start.to_vec2()) * Affine::rotate(angle)
}

/// Append a quarter-circle arc to the path using cubic bezier approximation.
///
/// `center`: arc center point
/// `radius`: arc radius
/// `start_angle`: angle where the arc starts (radians)
/// `sweep_angle`: angle to sweep (positive = clockwise in SVG coords)
fn append_quarter_arc(
    path: &mut BezPath,
    center: Point,
    radius: f64,
    start_angle: f64,
    sweep_angle: f64,
) {
    let arc = Arc::new(
        center,
        Vec2::new(radius, radius),
        start_angle,
        sweep_angle,
        0.0,
    );
    arc.to_cubic_beziers(0.01, |p1, p2, p3| {
        path.curve_to(p1, p2, p3);
    });
}

/// Wrap SVG path data in a complete SVG document.
///
/// Output format:
/// - Physical dimensions in mm (`width`, `height`)
/// - Matching `viewBox` for 1:1 mm coordinate mapping
/// - Single `<path>` element with hairline black stroke
/// - No fill, no metadata, no title/desc
fn build_svg_document(path_data: &str, width_mm: f64, height_mm: f64) -> String {
    format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='{w}mm' height='{h}mm' viewBox='0 0 {w} {h}'>\
         <path d='{d}' stroke='#000000' stroke-width='0.001mm' fill='none'/>\
         </svg>",
        w = width_mm,
        h = height_mm,
        d = path_data,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classic_connector::ClassicKnobConnector;
    use crate::config::*;

    /// Helper to create a valid PuzzleConfig for SVG export testing.
    fn test_config(rows: u32, cols: u32, seed: &str) -> PuzzleConfig {
        PuzzleConfig {
            rows,
            cols,
            width: 200.0,
            height: 150.0,
            unit: Unit::Millimeters,
            tab: TabConfig::default(),
            border: BorderConfig::default(),
            seed: seed.to_string(),
        }
    }

    /// Generate an SVG from a grid with connectors.
    fn generate_test_svg(rows: u32, cols: u32, seed: &str) -> String {
        let config = test_config(rows, cols, seed);
        let mut grid = PuzzleGrid::new(config).unwrap();
        grid.generate_connectors(&ClassicKnobConnector);
        generate_svg(&grid)
    }

    #[test]
    fn test_svg_contains_path_element() {
        let svg = generate_test_svg(3, 4, "path-test");
        assert!(svg.contains("<path"), "SVG should contain <path element");
    }

    #[test]
    fn test_svg_has_mm_dimensions() {
        let svg = generate_test_svg(3, 4, "mm-test");
        assert!(
            svg.contains("width='200mm'"),
            "SVG should have width in mm: {}",
            svg
        );
        assert!(
            svg.contains("height='150mm'"),
            "SVG should have height in mm: {}",
            svg
        );
    }

    #[test]
    fn test_svg_has_viewbox() {
        let svg = generate_test_svg(3, 4, "viewbox-test");
        assert!(
            svg.contains("viewBox='0 0 200 150'"),
            "SVG should have matching viewBox: {}",
            svg
        );
    }

    #[test]
    fn test_svg_has_stroke_attributes() {
        let svg = generate_test_svg(3, 4, "stroke-test");
        assert!(
            svg.contains("stroke='#000000'"),
            "SVG should have black stroke"
        );
        assert!(
            svg.contains("stroke-width='0.001mm'"),
            "SVG should have hairline stroke width"
        );
        assert!(svg.contains("fill='none'"), "SVG should have no fill");
    }

    #[test]
    fn test_svg_has_xmlns() {
        let svg = generate_test_svg(3, 4, "xmlns-test");
        assert!(
            svg.contains("xmlns='http://www.w3.org/2000/svg'"),
            "SVG should have xmlns attribute"
        );
    }

    #[test]
    fn test_svg_no_relative_commands() {
        let svg = generate_test_svg(3, 4, "abs-test");
        // Extract path data between d=' and '
        let d_start = svg.find("d='").expect("should have d attribute") + 3;
        let d_end = svg[d_start..].find('\'').unwrap() + d_start;
        let path_data = &svg[d_start..d_end];

        // Check for lowercase relative commands (m, l, c, z are relative)
        // Only uppercase M, L, C, Z should appear
        for ch in path_data.chars() {
            if ch.is_ascii_lowercase() && "mlcqz".contains(ch) {
                panic!(
                    "path data contains relative command '{}': should only use absolute M, L, C, Z",
                    ch
                );
            }
        }
    }

    #[test]
    fn test_svg_border_is_closed() {
        let svg = generate_test_svg(3, 4, "close-test");
        let d_start = svg.find("d='").unwrap() + 3;
        let d_end = svg[d_start..].find('\'').unwrap() + d_start;
        let path_data = &svg[d_start..d_end];
        assert!(
            path_data.contains('Z'),
            "path data should contain Z (closed border subpath)"
        );
    }

    #[test]
    fn test_svg_internal_edges_present() {
        let svg = generate_test_svg(3, 4, "internal-test");
        let d_start = svg.find("d='").unwrap() + 3;
        let d_end = svg[d_start..].find('\'').unwrap() + d_start;
        let path_data = &svg[d_start..d_end];

        // Count M commands — should be 1 (border) + N (internal edges)
        let m_count = path_data.matches('M').count();
        // 3x4 grid: internal h_edges = 2 rows * 4 cols = 8
        //           internal v_edges = 3 rows * 3 cols = 9
        //           total internal = 17 + 1 border = 18
        assert!(
            m_count > 1,
            "should have multiple M commands (border + internal edges), got {}",
            m_count
        );
        assert!(
            m_count >= 18,
            "3x4 grid should have at least 18 M commands (1 border + 17 internal edges), got {}",
            m_count
        );
    }

    #[test]
    fn test_svg_deterministic() {
        let svg1 = generate_test_svg(3, 4, "determ");
        let svg2 = generate_test_svg(3, 4, "determ");
        assert_eq!(svg1, svg2, "same config/seed must produce identical SVG");
    }

    #[test]
    fn test_svg_contains_cubic_curves() {
        let svg = generate_test_svg(3, 4, "curve-test");
        let d_start = svg.find("d='").unwrap() + 3;
        let d_end = svg[d_start..].find('\'').unwrap() + d_start;
        let path_data = &svg[d_start..d_end];
        assert!(
            path_data.contains('C'),
            "path data should contain C commands (cubic bezier curves)"
        );
    }

    #[test]
    fn test_edge_transform_horizontal() {
        let t = edge_transform(Point::new(50.0, 0.0), Point::new(100.0, 0.0));
        // For a horizontal edge, angle = 0, transform is just translation
        let p = t * Point::new(0.0, 0.0);
        assert!((p.x - 50.0).abs() < 1e-6 && (p.y - 0.0).abs() < 1e-6);

        let p = t * Point::new(50.0, 0.0);
        assert!((p.x - 100.0).abs() < 1e-6 && (p.y - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_edge_transform_vertical() {
        let t = edge_transform(Point::new(0.0, 50.0), Point::new(0.0, 100.0));
        // For a vertical downward edge, angle = PI/2
        let p = t * Point::new(0.0, 0.0);
        assert!((p.x - 0.0).abs() < 1e-6 && (p.y - 50.0).abs() < 1e-6);
    }
}
