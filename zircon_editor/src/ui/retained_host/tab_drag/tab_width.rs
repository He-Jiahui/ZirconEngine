use crate::ui::retained_host::measure_runtime_text_width;
use crate::ui::workbench::document_tabs::{
    document_tab_preferred_width_from_title_width, DOCUMENT_TAB_TITLE_FONT_SIZE,
};

const DOCK_TAB_TITLE_FONT_SIZE: f32 = 12.0;
const DOCK_TAB_CHROME_WIDTH: f32 = 30.0;
const DOCK_TAB_MIN_WIDTH: f32 = 68.0;

pub(crate) fn estimate_dock_tab_width(label: &str) -> f32 {
    (measure_runtime_text_width(label, DOCK_TAB_TITLE_FONT_SIZE) + DOCK_TAB_CHROME_WIDTH)
        .max(DOCK_TAB_MIN_WIDTH)
}

pub(crate) fn estimate_document_tab_width(label: &str, closeable: bool) -> f32 {
    let title_width = measure_runtime_text_width(label, DOCUMENT_TAB_TITLE_FONT_SIZE);
    document_tab_preferred_width_from_title_width(title_width, closeable)
}

#[cfg(test)]
mod tests {
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
}
