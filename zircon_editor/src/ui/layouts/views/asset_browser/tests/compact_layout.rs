use super::*;

#[test]
fn collapsed_vertical_budget_never_expands_a_short_viewport() {
    let (main_height, utility_y, utility_height) =
        compact_asset_browser_vertical_budget(NARROW_BREAKPOINT_WIDTH, 120.0, 96.0, 8.0);

    assert_eq!(main_height, 0.0);
    assert_eq!(utility_y, 96.0);
    assert_eq!(utility_height, 24.0);
}

#[test]
fn ultra_width_vertical_budget_reclaims_hidden_utility_space() {
    let (main_height, utility_y, utility_height) =
        compact_asset_browser_vertical_budget(NARROW_BREAKPOINT_WIDTH - 1.0, 720.0, 120.0, 8.0);

    assert_eq!(utility_y, 720.0);
    assert_eq!(utility_height, 0.0);
    assert_eq!(main_height, 592.0);
}

#[test]
fn invalid_viewport_values_collapse_to_zero_geometry() {
    assert_eq!(
        compact_asset_browser_utility_height_for_viewport(f32::NAN),
        0.0
    );
    assert_eq!(finite_non_negative(f32::INFINITY), 0.0);
    assert_eq!(finite_coordinate(f32::NEG_INFINITY), 0.0);
}

#[test]
fn compact_text_line_is_clipped_to_the_remaining_parent_height() {
    assert_eq!(compact_line_height(18.0, 12.0, 10.0), 6.0);
    assert_eq!(compact_line_height(8.0, 12.0, 10.0), 0.0);
}

#[test]
fn compact_details_keep_complete_typography_lines_or_hide_them() {
    assert_eq!(
        details_line_height(
            DETAILS_CAPTION_LINE_HEIGHT + DETAILS_TEXT_TOP,
            DETAILS_TEXT_TOP,
            DETAILS_CAPTION_LINE_HEIGHT,
        ),
        DETAILS_CAPTION_LINE_HEIGHT
    );
    assert_eq!(
        details_line_height(
            DETAILS_CAPTION_LINE_HEIGHT + DETAILS_TEXT_TOP - 0.5,
            DETAILS_TEXT_TOP,
            DETAILS_CAPTION_LINE_HEIGHT,
        ),
        0.0
    );
}

#[test]
fn compact_content_header_keeps_complete_typography_lines_or_hides_them() {
    assert_eq!(
        complete_text_line_height(
            COMPACT_CONTENT_TITLE_LINE_HEIGHT,
            COMPACT_CONTENT_TITLE_LINE_HEIGHT,
        ),
        COMPACT_CONTENT_TITLE_LINE_HEIGHT
    );
    assert_eq!(
        complete_text_line_height(
            COMPACT_CONTENT_PATH_LINE_HEIGHT - 0.5,
            COMPACT_CONTENT_PATH_LINE_HEIGHT,
        ),
        0.0
    );
}
