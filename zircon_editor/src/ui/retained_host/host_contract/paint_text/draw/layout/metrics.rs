const TOTAL_ADVANCE_TOLERANCE_PX: f32 = 1.0;
const TOTAL_ADVANCE_TOLERANCE_RATIO: f32 = 0.15;
const RETAINED_GLYPH_ADVANCE_TOLERANCE_PX: f32 = 0.0625;
const RETAINED_GLYPH_ORIGIN_VISUAL_TOLERANCE_PX: f32 = 0.03125;
const LINE_VERTICAL_CENTER_FACTOR: f32 = 0.5;
const EMPTY_RUNTIME_LINE_FRAME_X_PX: f32 = 0.0;
const EMPTY_GRAPHEME_ADVANCE_PX: f32 = 0.0;
const MISSING_GLYPH_LEFT_OFFSET_PX: f32 = 0.0;

pub(super) fn non_negative_advance(advance: f32) -> f32 {
    advance.max(0.0)
}

pub(super) fn centered_line_y(rect_y: f32, rect_height: f32, line_height: f32) -> f32 {
    rect_y + non_negative_advance(rect_height - line_height) * LINE_VERTICAL_CENTER_FACTOR
}

pub(super) fn empty_runtime_line_frame_x() -> f32 {
    EMPTY_RUNTIME_LINE_FRAME_X_PX
}

pub(super) fn empty_grapheme_advance_px() -> f32 {
    EMPTY_GRAPHEME_ADVANCE_PX
}

pub(super) fn missing_host_advance() -> f32 {
    EMPTY_GRAPHEME_ADVANCE_PX
}

pub(super) fn missing_glyph_left_offset_px() -> f32 {
    MISSING_GLYPH_LEFT_OFFSET_PX
}

pub(super) fn advances_include_positive_width(advances: &[f32]) -> bool {
    advances
        .iter()
        .any(|advance| *advance > EMPTY_GRAPHEME_ADVANCE_PX)
}

pub(super) fn total_advances_match(runtime_width: f32, host_width: f32) -> bool {
    runtime_width.is_finite()
        && host_width.is_finite()
        && runtime_width > 0.0
        && host_width > 0.0
        && (runtime_width - host_width).abs() <= total_advance_tolerance(host_width)
}

pub(super) fn grapheme_advances_match(runtime_advance: f32, host_advance: f32) -> bool {
    let runtime_advance = non_negative_advance(runtime_advance);
    let host_advance = non_negative_advance(host_advance);
    runtime_advance.is_finite()
        && host_advance.is_finite()
        && (runtime_advance - host_advance).abs() <= grapheme_advance_tolerance(host_advance)
}

pub(super) fn glyph_origin_preserves_monotonic_order(
    origin_x: f32,
    previous_origin: Option<f32>,
) -> bool {
    if !origin_x.is_finite() {
        return false;
    }
    match previous_origin {
        Some(previous_origin) => origin_x + RETAINED_GLYPH_ADVANCE_TOLERANCE_PX >= previous_origin,
        None => true,
    }
}

pub(super) fn glyph_origin_matches_without_visible_drift(
    host_origin: f32,
    candidate_origin: f32,
) -> bool {
    host_origin.is_finite()
        && candidate_origin.is_finite()
        && (host_origin - candidate_origin).abs() <= RETAINED_GLYPH_ORIGIN_VISUAL_TOLERANCE_PX
}

fn total_advance_tolerance(reference_width: f32) -> f32 {
    TOTAL_ADVANCE_TOLERANCE_PX
        .max(non_negative_advance(reference_width) * TOTAL_ADVANCE_TOLERANCE_RATIO)
}

fn grapheme_advance_tolerance(_host_advance: f32) -> f32 {
    RETAINED_GLYPH_ADVANCE_TOLERANCE_PX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_tolerance_keeps_absolute_floor_for_small_runs() {
        assert_eq!(total_advance_tolerance(4.0), TOTAL_ADVANCE_TOLERANCE_PX);
        assert!(total_advances_match(4.9, 4.0));
        assert!(!total_advances_match(5.2, 4.0));
    }

    #[test]
    fn total_tolerance_scales_for_wide_runs() {
        assert_eq!(total_advance_tolerance(20.0), 3.0);
        assert!(total_advances_match(22.9, 20.0));
        assert!(!total_advances_match(23.2, 20.0));
    }

    #[test]
    fn grapheme_advance_tolerance_keeps_phase_bin_guard() {
        assert!(grapheme_advances_match(8.03125, 8.0));
        assert!(!grapheme_advances_match(8.125, 8.0));
        assert_eq!(non_negative_advance(-2.0), 0.0);
    }

    #[test]
    fn glyph_origin_order_allows_subpixel_backtrack_only() {
        assert!(glyph_origin_preserves_monotonic_order(9.95, Some(10.0)));
        assert!(!glyph_origin_preserves_monotonic_order(9.9, Some(10.0)));
        assert!(glyph_origin_preserves_monotonic_order(10.0, None));
        assert!(!glyph_origin_preserves_monotonic_order(f32::NAN, None));
    }

    #[test]
    fn glyph_origin_drift_guard_rejects_visible_same_phase_offsets() {
        assert!(glyph_origin_matches_without_visible_drift(20.0, 20.03125));
        assert!(!glyph_origin_matches_without_visible_drift(20.0, 20.05));
        assert!(!glyph_origin_matches_without_visible_drift(f32::NAN, 20.0));
    }

    #[test]
    fn centered_line_y_uses_non_negative_extra_height() {
        assert_eq!(centered_line_y(10.0, 20.0, 12.0), 14.0);
        assert_eq!(centered_line_y(10.0, 8.0, 12.0), 10.0);
    }

    #[test]
    fn layout_position_defaults_are_named() {
        assert_eq!(empty_runtime_line_frame_x(), 0.0);
        assert_eq!(empty_grapheme_advance_px(), 0.0);
        assert_eq!(missing_host_advance(), 0.0);
        assert_eq!(missing_glyph_left_offset_px(), 0.0);
    }

    #[test]
    fn positive_advance_predicate_ignores_empty_or_negative_runs() {
        assert!(!advances_include_positive_width(&[0.0, -1.0]));
        assert!(advances_include_positive_width(&[0.0, 0.25]));
    }
}
