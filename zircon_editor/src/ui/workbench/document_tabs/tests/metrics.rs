use super::*;

#[test]
fn document_tab_typography_uses_workbench_body_role() {
    assert_eq!(
        DOCUMENT_TAB_TITLE_FONT_SIZE,
        EditorTypographyTokens::WORKBENCH_BODY_SIZE
    );
}

#[test]
fn closeable_document_tab_width_keeps_asset_browser_title_readable() {
    let width = document_tab_preferred_width_from_title_width(72.0, true);

    assert!(width >= DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH);
    assert!(width <= DOCUMENT_TAB_MAX_WIDTH);
}

#[test]
fn document_tab_width_clamps_measured_title_width() {
    assert_eq!(
        document_tab_preferred_width_from_title_width(1.0, false),
        DOCUMENT_TAB_MIN_WIDTH
    );
    assert_eq!(
        document_tab_preferred_width_from_title_width(10_000.0, true),
        DOCUMENT_TAB_MAX_WIDTH
    );
    assert_eq!(
        document_tab_preferred_width_from_title_width(f32::NAN, true),
        DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH
    );
}

#[test]
fn close_button_frame_uses_shared_right_inset_and_extent() {
    let tab_x = DOCUMENT_TAB_STRIP_X;
    let close_x = document_tab_close_x(tab_x, DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH);

    assert_eq!(
        close_x + DOCUMENT_TAB_CLOSE_EXTENT + DOCUMENT_TAB_CLOSE_RIGHT_INSET,
        tab_x + DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH
    );
}
