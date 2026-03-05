use kurbo::{CubicBez, ParamCurveExtrema, Point};
use rand_chacha::ChaCha8Rng;

use crate::connector::ConnectorGenerator;
use crate::edge::{EdgeParams, TabDirection};

/// Ratio of knob height to knob width (height = width * this).
const KNOB_HEIGHT_RATIO: f64 = 1.2;

// Neck width ratio is now dynamic via params.neck_ratio (from TabConfig.taper).
// taper=0.0 → neck_ratio=1.0, taper=0.5 → 0.75 (classic), taper=1.0 → 0.5.

/// Neck height as a fraction of knob height (how tall the neck is before widening).
const NECK_HEIGHT_RATIO: f64 = 0.35;

/// How far beyond the knob top the control points overshoot (creates rounded top).
const TOP_OVERSHOOT: f64 = 1.05;

/// Width of the knob body as a fraction of knob_w (where the body meets the top curve).
const BODY_WIDTH_RATIO: f64 = 0.3;

/// How much of the knob height the body transition covers.
const BODY_SHOULDER_RATIO: f64 = 0.85;

/// How much of the knob height the neck-to-body transition covers.
const NECK_BODY_RATIO: f64 = 0.6;

/// How far the first control point extends beyond the knob entry (approach angle).
const APPROACH_RATIO: f64 = 1.2;

/// Width of the knob top curve control points (how wide the rounded top is).
const TOP_WIDTH_RATIO: f64 = 0.1;

/// Classic jigsaw knob connector generator.
///
/// Produces traditional Ravensburger-style knob shapes using cubic bezier curves
/// in edge-local coordinates. Each knob has a visible neck (narrower than the body)
/// for interlocking snap-fit of laser-cut pieces.
///
/// ## Knob Anatomy (edge-local, direction=Out)
///
/// ```text
/// Edge baseline: y=0, x from 0 to length
/// Knob center: x = length/2, y = knob_h
///
///   [flat]──[neck-in]──[body-out]──[top-round]──[body-out]──[neck-in]──[flat]
///            narrowing   widening    rounded top   narrowing
///
/// The neck narrowing is what makes laser-cut pieces snap together.
/// ```
///
/// The shape consists of 5 cubic bezier segments:
/// 1. Baseline → neck entry (left side)
/// 2. Neck → knob body (widening)
/// 3. Knob top (rounded bell curve)
/// 4. Knob body → neck (narrowing, mirror of segment 2)
/// 5. Neck exit → baseline (right side)
pub struct ClassicKnobConnector;

impl ConnectorGenerator for ClassicKnobConnector {
    fn generate(&self, params: &EdgeParams, _rng: &mut ChaCha8Rng) -> Vec<CubicBez> {
        let length = params.length;

        // Direction sign: +1.0 for Out (knob extends in +Y), -1.0 for In
        let dir_sign = match params.direction {
            TabDirection::Out => 1.0,
            TabDirection::In => -1.0,
        };

        // Knob is always centered on the edge
        let center = length * 0.5;

        // Knob dimensions
        let knob_w = length * params.tab_size;
        let knob_h = knob_w * KNOB_HEIGHT_RATIO * dir_sign;
        let neck_w = knob_w * params.neck_ratio;
        let neck_h = knob_h * NECK_HEIGHT_RATIO;

        // Build 5 cubic bezier segments
        vec![
            // 1. Baseline → neck entry (left side)
            CubicBez::new(
                Point::new(0.0, 0.0),
                Point::new(center - knob_w * APPROACH_RATIO, 0.0),
                Point::new(center - neck_w, 0.0),
                Point::new(center - neck_w, neck_h),
            ),
            // 2. Neck → knob body (left side, widening)
            CubicBez::new(
                Point::new(center - neck_w, neck_h),
                Point::new(center - neck_w, knob_h * NECK_BODY_RATIO),
                Point::new(center - knob_w, knob_h * BODY_SHOULDER_RATIO),
                Point::new(center - knob_w * BODY_WIDTH_RATIO, knob_h),
            ),
            // 3. Knob top (rounded)
            CubicBez::new(
                Point::new(center - knob_w * BODY_WIDTH_RATIO, knob_h),
                Point::new(center - knob_w * TOP_WIDTH_RATIO, knob_h * TOP_OVERSHOOT),
                Point::new(center + knob_w * TOP_WIDTH_RATIO, knob_h * TOP_OVERSHOOT),
                Point::new(center + knob_w * BODY_WIDTH_RATIO, knob_h),
            ),
            // 4. Knob body → neck (right side, narrowing)
            CubicBez::new(
                Point::new(center + knob_w * BODY_WIDTH_RATIO, knob_h),
                Point::new(center + knob_w, knob_h * BODY_SHOULDER_RATIO),
                Point::new(center + neck_w, knob_h * NECK_BODY_RATIO),
                Point::new(center + neck_w, neck_h),
            ),
            // 5. Neck exit → baseline (right side)
            CubicBez::new(
                Point::new(center + neck_w, neck_h),
                Point::new(center + neck_w, 0.0),
                Point::new(center + knob_w * APPROACH_RATIO, 0.0),
                Point::new(length, 0.0),
            ),
        ]
    }

    fn validate(&self, curves: &[CubicBez], params: &EdgeParams) -> Result<(), String> {
        if curves.is_empty() {
            return Err("no curves to validate".to_string());
        }

        // Check first point is at origin
        let first_p0 = curves[0].p0;
        if first_p0.x.abs() > 1e-6 || first_p0.y.abs() > 1e-6 {
            return Err(format!(
                "first curve p0 should be (0,0), got ({}, {})",
                first_p0.x, first_p0.y
            ));
        }

        // Check last point is at (length, 0)
        let last_p3 = curves[curves.len() - 1].p3;
        if (last_p3.x - params.length).abs() > 1e-6 || last_p3.y.abs() > 1e-6 {
            return Err(format!(
                "last curve p3 should be ({}, 0), got ({}, {})",
                params.length, last_p3.x, last_p3.y
            ));
        }

        // Check continuity between segments
        for i in 1..curves.len() {
            let prev_end = curves[i - 1].p3;
            let curr_start = curves[i].p0;
            let gap =
                ((prev_end.x - curr_start.x).powi(2) + (prev_end.y - curr_start.y).powi(2)).sqrt();
            if gap > 1e-6 {
                return Err(format!(
                    "gap between curve {} and {}: {} (max 1e-6)",
                    i - 1,
                    i,
                    gap
                ));
            }
        }

        // Check bounding box doesn't exceed 5% beyond nominal piece boundary
        let margin = params.length * 0.05;
        for (i, curve) in curves.iter().enumerate() {
            let bbox = curve.bounding_box();
            if bbox.x0 < -margin
                || bbox.x1 > params.length + margin
                || bbox.y0 < -params.length - margin
                || bbox.y1 > params.length + margin
            {
                return Err(format!(
                    "curve {} bounding box ({:?}) exceeds 5% margin",
                    i, bbox
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::TabDirection;
    use crate::seed::create_rng;

    fn default_params(direction: TabDirection) -> EdgeParams {
        EdgeParams {
            length: 50.0,
            cross_length: 50.0,
            direction,
            tab_size: 0.25,
            neck_ratio: 0.75,
        }
    }

    // ─── Shape Tests ──────────────────────────────────────────────

    #[test]
    fn test_generates_nonzero_curves() {
        let connector = ClassicKnobConnector;
        let params = default_params(TabDirection::Out);
        let mut rng = create_rng("test");
        let curves = connector.generate(&params, &mut rng);
        assert!(
            !curves.is_empty(),
            "generate() must return at least one curve"
        );
    }

    #[test]
    fn test_curves_start_at_origin() {
        let connector = ClassicKnobConnector;
        let params = default_params(TabDirection::Out);
        let mut rng = create_rng("origin-test");
        let curves = connector.generate(&params, &mut rng);
        assert!(!curves.is_empty(), "need curves to test");
        let first = &curves[0];
        assert!(
            (first.p0.x).abs() < 1e-10,
            "first curve p0.x should be 0, got {}",
            first.p0.x
        );
        assert!(
            (first.p0.y).abs() < 1e-10,
            "first curve p0.y should be 0, got {}",
            first.p0.y
        );
    }

    #[test]
    fn test_curves_end_at_edge_end() {
        let connector = ClassicKnobConnector;
        let params = default_params(TabDirection::Out);
        let mut rng = create_rng("end-test");
        let curves = connector.generate(&params, &mut rng);
        assert!(!curves.is_empty(), "need curves to test");
        let last = &curves[curves.len() - 1];
        assert!(
            (last.p3.x - params.length).abs() < 1e-10,
            "last curve p3.x should be {}, got {}",
            params.length,
            last.p3.x
        );
        assert!(
            (last.p3.y).abs() < 1e-10,
            "last curve p3.y should be 0, got {}",
            last.p3.y
        );
    }

    #[test]
    fn test_curves_continuous() {
        let connector = ClassicKnobConnector;
        let params = default_params(TabDirection::Out);
        let mut rng = create_rng("continuity-test");
        let curves = connector.generate(&params, &mut rng);
        assert!(curves.len() >= 2, "need at least 2 curves");
        for i in 1..curves.len() {
            let prev_end = curves[i - 1].p3;
            let curr_start = curves[i].p0;
            let gap =
                ((prev_end.x - curr_start.x).powi(2) + (prev_end.y - curr_start.y).powi(2)).sqrt();
            assert!(
                gap < 1e-6,
                "gap between curve {} and {} is {}, should be < 1e-6",
                i - 1,
                i,
                gap
            );
        }
    }

    // ─── Direction Tests ──────────────────────────────────────────

    #[test]
    fn test_direction_out_positive_y() {
        let connector = ClassicKnobConnector;
        let params = default_params(TabDirection::Out);
        let mut rng = create_rng("dir-out");
        let curves = connector.generate(&params, &mut rng);
        let has_positive_y = curves
            .iter()
            .any(|c| c.p0.y > 1e-6 || c.p1.y > 1e-6 || c.p2.y > 1e-6 || c.p3.y > 1e-6);
        assert!(
            has_positive_y,
            "TabDirection::Out should produce control points with y > 0"
        );
    }

    #[test]
    fn test_direction_in_negative_y() {
        let connector = ClassicKnobConnector;
        let params = default_params(TabDirection::In);
        let mut rng = create_rng("dir-in");
        let curves = connector.generate(&params, &mut rng);
        let has_negative_y = curves
            .iter()
            .any(|c| c.p0.y < -1e-6 || c.p1.y < -1e-6 || c.p2.y < -1e-6 || c.p3.y < -1e-6);
        assert!(
            has_negative_y,
            "TabDirection::In should produce control points with y < 0"
        );
    }

    // ─── Center Tests ─────────────────────────────────────────────

    #[test]
    fn test_knob_centered_on_edge() {
        let connector = ClassicKnobConnector;
        let params = default_params(TabDirection::Out);
        let mut rng = create_rng("center-test");
        let curves = connector.generate(&params, &mut rng);
        assert!(!curves.is_empty(), "need curves");

        // Knob top (segment 2) should be centered at length/2
        let mid_curve = &curves[curves.len() / 2];
        let mid_x = (mid_curve.p0.x + mid_curve.p3.x) / 2.0;
        assert!(
            (mid_x - params.length / 2.0).abs() < 1e-6,
            "knob center x should be at {}, got {}",
            params.length / 2.0,
            mid_x
        );
    }

    #[test]
    fn test_identical_shape_across_seeds() {
        let connector = ClassicKnobConnector;
        let params = default_params(TabDirection::Out);

        let mut rng1 = create_rng("seed-a");
        let curves1 = connector.generate(&params, &mut rng1);

        let mut rng2 = create_rng("seed-b");
        let curves2 = connector.generate(&params, &mut rng2);

        // Same params → identical curves regardless of RNG seed
        for (c1, c2) in curves1.iter().zip(curves2.iter()) {
            assert!(
                (c1.p0.x - c2.p0.x).abs() < 1e-10
                    && (c1.p0.y - c2.p0.y).abs() < 1e-10
                    && (c1.p1.x - c2.p1.x).abs() < 1e-10
                    && (c1.p1.y - c2.p1.y).abs() < 1e-10
                    && (c1.p2.x - c2.p2.x).abs() < 1e-10
                    && (c1.p2.y - c2.p2.y).abs() < 1e-10
                    && (c1.p3.x - c2.p3.x).abs() < 1e-10
                    && (c1.p3.y - c2.p3.y).abs() < 1e-10,
                "curves should be identical regardless of RNG seed"
            );
        }
    }

    // ─── Proportion Tests ─────────────────────────────────────────

    #[test]
    fn test_tab_size_affects_proportions() {
        let connector = ClassicKnobConnector;

        let small_params = EdgeParams {
            length: 50.0,
            cross_length: 50.0,
            direction: TabDirection::Out,
            tab_size: 0.15,
            neck_ratio: 0.75,
        };

        let large_params = EdgeParams {
            length: 50.0,
            cross_length: 50.0,
            direction: TabDirection::Out,
            tab_size: 0.25,
            neck_ratio: 0.75,
        };

        let mut rng1 = create_rng("size-test");
        let small_curves = connector.generate(&small_params, &mut rng1);
        let mut rng2 = create_rng("size-test");
        let large_curves = connector.generate(&large_params, &mut rng2);

        assert!(!small_curves.is_empty() && !large_curves.is_empty());

        // Max y extent of large tab should be bigger than small tab
        let max_y_small = small_curves
            .iter()
            .flat_map(|c| [c.p0.y, c.p1.y, c.p2.y, c.p3.y])
            .fold(0.0f64, f64::max);
        let max_y_large = large_curves
            .iter()
            .flat_map(|c| [c.p0.y, c.p1.y, c.p2.y, c.p3.y])
            .fold(0.0f64, f64::max);

        assert!(
            max_y_large > max_y_small,
            "larger tab_size should produce larger y extent: small={}, large={}",
            max_y_small,
            max_y_large
        );
    }

    // ─── Validation Tests ─────────────────────────────────────────

    #[test]
    fn test_validate_accepts_good_curves() {
        let connector = ClassicKnobConnector;
        let params = default_params(TabDirection::Out);
        let mut rng = create_rng("validate-test");
        let curves = connector.generate(&params, &mut rng);
        assert!(!curves.is_empty(), "need curves to validate");
        let result = connector.validate(&curves, &params);
        assert!(
            result.is_ok(),
            "validate should accept generated curves: {:?}",
            result
        );
    }

    // ─── Neck Tests ───────────────────────────────────────────────

    #[test]
    fn test_has_visible_neck() {
        let connector = ClassicKnobConnector;
        let params = EdgeParams {
            length: 50.0,
            cross_length: 50.0,
            direction: TabDirection::Out,
            tab_size: 0.25,
            neck_ratio: 0.75,
        };
        let mut rng = create_rng("neck-test");
        let curves = connector.generate(&params, &mut rng);
        assert!(curves.len() >= 5, "expect at least 5 segments");

        // The neck segments (1st and 4th from ends) should have smaller y extent
        // than the body segments (middle).
        // Segment 0: baseline to neck entry (near baseline)
        // Segment 1: neck widening
        // Segment 2: knob top (max height)
        // Segment 3: narrowing
        // Segment 4: neck exit to baseline

        // The top of the knob (segment 2) should have larger y than neck entries
        let knob_top_y = [
            curves[2].p0.y,
            curves[2].p1.y,
            curves[2].p2.y,
            curves[2].p3.y,
        ]
        .iter()
        .copied()
        .fold(0.0f64, f64::max);

        // The neck entry y (end of segment 0) should be smaller than knob top
        let neck_entry_y = curves[0].p3.y;

        assert!(
            knob_top_y > neck_entry_y,
            "knob top y ({}) should be higher than neck entry y ({})",
            knob_top_y,
            neck_entry_y
        );
    }
}
