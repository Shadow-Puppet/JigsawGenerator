use kurbo::CubicBez;
use rand_chacha::ChaCha8Rng;

use crate::connector::ConnectorGenerator;
use crate::edge::EdgeParams;

/// Classic jigsaw knob connector generator.
///
/// Produces traditional Ravensburger-style knob shapes using cubic bezier curves
/// in edge-local coordinates. Each knob has a visible neck (narrower than the body)
/// for interlocking snap-fit of laser-cut pieces.
///
/// The knob shape consists of 5 cubic bezier segments:
/// 1. Baseline to neck entry (left side)
/// 2. Neck to knob body (widening)
/// 3. Knob top (rounded bell curve)
/// 4. Knob body to neck (narrowing, mirror of segment 2)
/// 5. Neck exit to baseline (right side)
pub struct ClassicKnobConnector;

impl ConnectorGenerator for ClassicKnobConnector {
    fn generate(&self, _params: &EdgeParams, _rng: &mut ChaCha8Rng) -> Vec<CubicBez> {
        // Stub: returns empty vec (tests will fail against this)
        Vec::new()
    }

    fn validate(&self, _curves: &[CubicBez], _params: &EdgeParams) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::TabDirection;
    use crate::seed::create_rng;
    use kurbo::Point;

    fn default_params(direction: TabDirection) -> EdgeParams {
        EdgeParams {
            length: 50.0,
            direction,
            tab_size: 0.25,
            jitter_amount: 0.5,
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

    // ─── Variation Tests ──────────────────────────────────────────

    #[test]
    fn test_jitter_produces_variation() {
        let connector = ClassicKnobConnector;
        let params = default_params(TabDirection::Out);

        let mut rng1 = create_rng("jitter-a");
        let curves1 = connector.generate(&params, &mut rng1);

        let mut rng2 = create_rng("jitter-b");
        let curves2 = connector.generate(&params, &mut rng2);

        assert!(!curves1.is_empty() && !curves2.is_empty(), "need curves");

        // At least one control point should differ
        let any_diff = curves1.iter().zip(curves2.iter()).any(|(c1, c2)| {
            (c1.p1.x - c2.p1.x).abs() > 1e-10
                || (c1.p1.y - c2.p1.y).abs() > 1e-10
                || (c1.p2.x - c2.p2.x).abs() > 1e-10
                || (c1.p2.y - c2.p2.y).abs() > 1e-10
        });
        assert!(
            any_diff,
            "Different RNG states should produce different control points"
        );
    }

    #[test]
    fn test_zero_jitter_deterministic() {
        let connector = ClassicKnobConnector;
        let params = EdgeParams {
            length: 50.0,
            direction: TabDirection::Out,
            tab_size: 0.25,
            jitter_amount: 0.0,
        };

        let mut rng = create_rng("zero-jitter");
        let curves = connector.generate(&params, &mut rng);
        assert!(!curves.is_empty(), "need curves");

        // With zero jitter, knob center should be at exactly length/2
        // Check: the midpoint of all curves' x range should be centered
        // More specifically: the 3rd curve (knob top) should be centered at length/2
        let mid_curve = &curves[curves.len() / 2];
        let mid_x = (mid_curve.p0.x + mid_curve.p3.x) / 2.0;
        assert!(
            (mid_x - params.length / 2.0).abs() < 1e-6,
            "with zero jitter, knob center x should be at {}, got {}",
            params.length / 2.0,
            mid_x
        );
    }

    // ─── Proportion Tests ─────────────────────────────────────────

    #[test]
    fn test_tab_size_affects_proportions() {
        let connector = ClassicKnobConnector;

        let small_params = EdgeParams {
            length: 50.0,
            direction: TabDirection::Out,
            tab_size: 0.15,
            jitter_amount: 0.0,
        };

        let large_params = EdgeParams {
            length: 50.0,
            direction: TabDirection::Out,
            tab_size: 0.45,
            jitter_amount: 0.0,
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
            direction: TabDirection::Out,
            tab_size: 0.25,
            jitter_amount: 0.0,
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
