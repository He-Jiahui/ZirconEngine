use super::*;

#[test]
fn main_page_tab_typography_uses_workbench_body_role() {
    assert_eq!(
        MAIN_PAGE_TAB_TITLE_FONT_SIZE,
        EditorTypographyTokens::WORKBENCH_BODY_SIZE
    );
}

#[test]
fn main_page_tab_width_clamps_measured_title_width() {
    assert_eq!(
        main_page_tab_preferred_width_from_title_width(1.0),
        MAIN_PAGE_TAB_MIN_WIDTH
    );
    assert_eq!(
        main_page_tab_preferred_width_from_title_width(10_000.0),
        MAIN_PAGE_TAB_MAX_WIDTH
    );
    assert_eq!(
        main_page_tab_preferred_width_from_title_width(f32::NAN),
        MAIN_PAGE_TAB_MIN_WIDTH
    );
}

#[test]
fn closeable_page_tabs_reserve_a_bounded_close_hit_target() {
    let plain = main_page_tab_preferred_width_from_title_width_with_close(96.0, false);
    let closeable = main_page_tab_preferred_width_from_title_width_with_close(96.0, true);
    let tab = zircon_runtime_interface::ui::layout::UiFrame::new(
        12.0,
        25.0,
        closeable,
        MAIN_PAGE_TAB_HEIGHT,
    );
    let close = main_page_tab_close_frame(tab);

    assert!(closeable > plain);
    assert!(closeable <= MAIN_PAGE_TAB_MAX_WIDTH);
    assert_eq!(close.width, MAIN_PAGE_TAB_CLOSE_EXTENT);
    assert_eq!(close.height, MAIN_PAGE_TAB_CLOSE_EXTENT);
    assert!(close.x >= tab.x);
    assert!(close.x + close.width <= tab.x + tab.width);
    assert!(close.y >= tab.y);
    assert!(close.y + close.height <= tab.y + tab.height);
}

#[test]
fn project_path_collapses_before_it_competes_with_primary_tabs() {
    assert_eq!(main_page_project_path_width(0.0), 0.0);
    assert_eq!(main_page_project_path_width(280.0), 0.0);
    assert_eq!(main_page_project_path_width(640.0), 150.0);
    assert_eq!(main_page_project_path_width(1260.0), 260.0);
}
