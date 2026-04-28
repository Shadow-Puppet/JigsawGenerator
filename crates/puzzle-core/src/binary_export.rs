//! Command-prefixed binary format constants used by
//! [`crate::layout`] export functions.
//!
//! The layout-based path serializer emits a flat `f64` array where each
//! command is a tagged record:
//!
//! - `CMD_MOVE_TO` (0.0) + `x`, `y` (3 floats total)
//! - `CMD_LINE_TO` (1.0) + `x`, `y` (3 floats)
//! - `CMD_CURVE_TO` (2.0) + `p1.x`, `p1.y`, `p2.x`, `p2.y`, `p3.x`, `p3.y` (7 floats)
//! - `CMD_CLOSE` (3.0) (1 float)
//!
//! This is the sole edge/border encoding shipped to JS — the fixed
//! 36-float stride format used by the old rectangular-only path has
//! been retired in favour of the unified command stream.

pub const CMD_MOVE_TO: f64 = 0.0;
pub const CMD_LINE_TO: f64 = 1.0;
pub const CMD_CURVE_TO: f64 = 2.0;
pub const CMD_CLOSE: f64 = 3.0;

/// Convert an arbitrary [`kurbo::BezPath`] to the command-prefixed
/// `f64` stream used by the canvas renderer (see module docs above).
/// Quadratic Béziers, if present, are elevated to cubic so the output
/// only uses `MoveTo/LineTo/CurveTo/Close` commands.
pub fn bezpath_to_binary(path: &kurbo::BezPath) -> Vec<f64> {
    use kurbo::PathEl;
    let mut data: Vec<f64> = Vec::with_capacity(path.elements().len() * 3);
    let mut last = kurbo::Point::ZERO;
    for el in path.elements() {
        match *el {
            PathEl::MoveTo(p) => {
                data.push(CMD_MOVE_TO);
                data.push(p.x);
                data.push(p.y);
                last = p;
            }
            PathEl::LineTo(p) => {
                data.push(CMD_LINE_TO);
                data.push(p.x);
                data.push(p.y);
                last = p;
            }
            PathEl::QuadTo(q, p) => {
                // Elevate to cubic so the canvas renderer only needs to
                // handle MoveTo/LineTo/CurveTo/Close.
                let cp1 = last + (q - last) * (2.0 / 3.0);
                let cp2 = p + (q - p) * (2.0 / 3.0);
                data.push(CMD_CURVE_TO);
                data.push(cp1.x);
                data.push(cp1.y);
                data.push(cp2.x);
                data.push(cp2.y);
                data.push(p.x);
                data.push(p.y);
                last = p;
            }
            PathEl::CurveTo(p1, p2, p3) => {
                data.push(CMD_CURVE_TO);
                data.push(p1.x);
                data.push(p1.y);
                data.push(p2.x);
                data.push(p2.y);
                data.push(p3.x);
                data.push(p3.y);
                last = p3;
            }
            PathEl::ClosePath => {
                data.push(CMD_CLOSE);
            }
        }
    }
    data
}
