use super::*;
use crate::ui::workbench::document_tabs::DOCUMENT_TAB_MAX_WIDTH;

#[test]
fn document_tab_drag_width_uses_runtime_text_measurement() {
    let label = "folder-open-line.svg";
    let expected = document_tab_preferred_width_from_title_width(
        measure_runtime_text_width(label, DOCUMENT_TAB_TITLE_FONT_SIZE),
        true,
    );

    assert_eq!(estimate_document_tab_width(label, true), expected);
    assert!(estimate_document_tab_width(label, true) <= DOCUMENT_TAB_MAX_WIDTH);
}

#[test]
fn dock_tab_drag_width_tracks_wide_and_narrow_runtime_glyphs() {
    let narrow = estimate_dock_tab_width("iiiiiiii");
    let wide = estimate_dock_tab_width("WWWWWWWW");

    assert!(
        wide > narrow,
        "runtime glyph measurement should keep wide labels wider than narrow labels"
    );
    assert!(narrow >= DOCK_TAB_MIN_WIDTH);
}
