use crate::ui::workbench::autolayout::{
    workbench_layout_tier_for_logical_width, WorkbenchLayoutTier,
};
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

pub(crate) const MAIN_PAGE_TAB_STRIP_X: f32 = 8.0;
pub(crate) const MAIN_PAGE_TAB_STRIP_Y: f32 = 1.0;
pub(crate) const MAIN_PAGE_TAB_MIN_WIDTH: f32 = 108.0;
pub(crate) const MAIN_PAGE_TAB_MAX_WIDTH: f32 = 180.0;
pub(crate) const MAIN_PAGE_TAB_HEIGHT: f32 = 30.0;
pub(crate) const MAIN_PAGE_TAB_GAP: f32 = 4.0;
pub(crate) const MAIN_PAGE_TAB_OVERFLOW_WIDTH: f32 = 36.0;
pub(crate) const MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH: f32 = 172.0;
pub(crate) const MAIN_PAGE_TAB_CHROME_SIDE_INSET: f32 = 12.0;
pub(crate) const MAIN_PAGE_TAB_TITLE_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE;

const TITLE_CHROME_RESERVE: f32 = 38.0;
const PROJECT_PATH_WIDTH_RATIO: f32 = 0.22;
const PROJECT_PATH_MIN_WIDTH: f32 = 150.0;
const PROJECT_PATH_MAX_WIDTH: f32 = 260.0;
const NARROW_VISIBLE_TAB_CAP: usize = 2;

pub(crate) fn main_page_tab_preferred_width_from_title_width(title_width: f32) -> f32 {
    let title_width = if title_width.is_finite() {
        title_width.max(0.0)
    } else {
        0.0
    };

    (title_width + TITLE_CHROME_RESERVE).clamp(MAIN_PAGE_TAB_MIN_WIDTH, MAIN_PAGE_TAB_MAX_WIDTH)
}

pub(crate) fn main_page_project_path_width(shell_width: f32) -> f32 {
    shell_width
        .max(0.0)
        .mul_add(PROJECT_PATH_WIDTH_RATIO, 0.0)
        .clamp(PROJECT_PATH_MIN_WIDTH, PROJECT_PATH_MAX_WIDTH)
}

pub(crate) fn main_page_tab_visible_cap_for_width(width: f32, page_count: usize) -> usize {
    if page_count == 0 {
        return 0;
    }
    match workbench_layout_tier_for_logical_width(width) {
        WorkbenchLayoutTier::Ultra | WorkbenchLayoutTier::Narrow => {
            page_count.min(NARROW_VISIBLE_TAB_CAP).max(1)
        }
        WorkbenchLayoutTier::Regular | WorkbenchLayoutTier::Wide => page_count,
    }
}

#[cfg(test)]
mod tests {
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
}
