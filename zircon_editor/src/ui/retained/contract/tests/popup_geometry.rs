use super::*;

#[test]
fn overflow_popup_width_tracks_runtime_title_measure_inside_shell() {
    let shell_width = 640.0;
    let title = "A long hidden editor tab title that should keep its useful glyph space";
    let presentation = overflow_presentation(shell_width, title);

    let popup = host_page_overflow_popup_frame(&presentation)
        .expect("open overflow should provide a popup frame");
    let metrics = current_host_metrics();
    let expected_width =
        (host_page_overflow_title_width(title, metrics.font_body, metrics.text_clip_guard)
            + MENU_POPUP_EDGE_INSET * 2.0
            + MENU_POPUP_TEXT_INSET_X * 2.0
            + metrics.selection_indicator_width
            + metrics.gap_s)
            .max(MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH);

    assert!(popup.width > MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH);
    assert_eq!(popup.width, expected_width);
    assert!(popup.x >= MENU_POPUP_SHELL_MARGIN);
    assert!(popup.x + popup.width <= shell_width - MENU_POPUP_SHELL_MARGIN);
}

#[test]
fn overflow_popup_width_constrains_itself_before_crossing_a_narrow_shell() {
    let shell_width = 96.0;
    let presentation = overflow_presentation(
        shell_width,
        "A long hidden editor tab title that cannot fit in this shell",
    );

    let popup = host_page_overflow_popup_frame(&presentation)
        .expect("open overflow should provide a popup frame");

    assert_eq!(popup.width, shell_width - menu_popup_shell_padding());
    assert_eq!(popup.x, MENU_POPUP_SHELL_MARGIN);
    assert_eq!(popup.x + popup.width, shell_width - MENU_POPUP_SHELL_MARGIN);
}

#[test]
fn overflow_popup_constrains_itself_to_an_offset_shell_frame() {
    let shell_x = 20.0;
    let shell_width = 640.0;
    let mut presentation = overflow_presentation(shell_width, "Hidden tab");
    presentation.host_layout.status_bar_frame.x = shell_x;
    presentation.host_layout.center_band_frame.x = shell_x;
    presentation.host_layout.center_band_frame.width = shell_width;
    presentation.host_scene_data.page_chrome.overflow_frame.x += shell_x;

    let popup = host_page_overflow_popup_frame(&presentation)
        .expect("offset shell should provide a relative popup frame");

    assert!(popup.x >= shell_x + MENU_POPUP_SHELL_MARGIN);
    assert!(
        popup.x + popup.width <= shell_x + shell_width - MENU_POPUP_SHELL_MARGIN,
        "popup right edge should use shell x + width instead of treating width as an absolute x"
    );
}

#[test]
fn overflow_popup_width_does_not_invent_a_one_pixel_shell() {
    let presentation = overflow_presentation(96.0, "Hidden");

    assert_eq!(
        host_page_overflow_popup_width(&presentation, menu_popup_shell_padding(), 0.0),
        0.0
    );
}

#[test]
fn overflow_long_list_natural_width_reserves_its_scrollbar_and_gap() {
    let shell_width = 640.0;
    let mut presentation = overflow_presentation(shell_width, "A naturally measured tab title");
    let unscrolled_width = host_page_overflow_popup_frame(&presentation)
        .expect("short list should provide a popup")
        .width;
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = vec![0; 32];
    let scrolled_width = host_page_overflow_popup_frame(&presentation)
        .expect("long list should provide a bounded popup")
        .width;
    let metrics = current_host_metrics();

    assert_eq!(
        scrolled_width,
        unscrolled_width + metrics.scrollbar_thickness + metrics.gap_s
    );
}

#[test]
fn overflow_natural_width_consumes_the_projected_widest_title_cache() {
    let shell_width = 640.0;
    let mut presentation = overflow_presentation(shell_width, "Short");
    presentation
        .host_scene_data
        .page_chrome
        .overflow_widest_title_width_px = 300.0;
    let metrics = current_host_metrics();
    let expected = 300.0
        + MENU_POPUP_EDGE_INSET * 2.0
        + MENU_POPUP_TEXT_INSET_X * 2.0
        + metrics.selection_indicator_width
        + metrics.gap_s;

    assert_eq!(
        host_page_overflow_popup_frame(&presentation)
            .expect("projected natural width should fit the shell")
            .width,
        expected
    );
}

#[test]
fn overflow_popup_rejects_a_non_finite_anchor_before_layout() {
    let mut presentation = overflow_presentation(240.0, "Hidden");
    presentation.host_scene_data.page_chrome.overflow_frame.x = f32::NAN;

    assert!(host_page_overflow_popup_frame(&presentation).is_none());
}

#[test]
fn overflow_title_width_uses_runtime_text_at_the_supplied_body_size() {
    let text = "WWWW";
    let body_size = EditorTypographyTokens::WORKBENCH_BODY_SIZE;

    assert_eq!(
        host_page_overflow_title_width(text, body_size, 6.0),
        measure_runtime_text_width(text, body_size) + 6.0
    );
}

#[test]
fn overflow_popup_uses_the_space_below_its_anchor_when_it_fits() {
    let placement = host_page_overflow_vertical_placement(
        &FrameRect {
            x: 0.0,
            y: 24.0,
            width: 34.0,
            height: 28.0,
        },
        160.0,
        80.0,
    );

    assert_eq!(placement.y, 55.0);
    assert_eq!(placement.height, 80.0);
}

#[test]
fn overflow_popup_flips_above_when_the_below_side_cannot_hold_its_content() {
    let placement = host_page_overflow_vertical_placement(
        &FrameRect {
            x: 0.0,
            y: 200.0,
            width: 34.0,
            height: 28.0,
        },
        240.0,
        80.0,
    );

    assert_eq!(placement.y, 117.0);
    assert_eq!(placement.height, 80.0);
}

#[test]
fn overflow_popup_clamps_its_viewport_inside_a_tiny_shell_without_overlapping_its_anchor() {
    let placement = host_page_overflow_vertical_placement(
        &FrameRect {
            x: 0.0,
            y: 24.0,
            width: 34.0,
            height: 28.0,
        },
        88.0,
        640.0,
    );

    assert_eq!(placement.y, 55.0);
    assert_eq!(placement.height, 25.0);
    assert!(placement.y + placement.height <= 80.0);
}

#[test]
fn overflow_popup_preserves_its_content_height_when_no_shell_bottom_is_available() {
    let placement = host_page_overflow_vertical_placement(
        &FrameRect {
            x: 0.0,
            y: 24.0,
            width: 34.0,
            height: 28.0,
        },
        0.0,
        80.0,
    );

    assert_eq!(placement.y, 55.0);
    assert_eq!(placement.height, 80.0);
}
