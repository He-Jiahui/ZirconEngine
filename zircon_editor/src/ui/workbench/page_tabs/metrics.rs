use crate::ui::workbench::autolayout::{
    workbench_layout_tier_for_logical_width, WorkbenchLayoutTier,
};
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;
use zircon_runtime_interface::ui::layout::UiFrame;

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
pub(crate) const MAIN_PAGE_TAB_CLOSE_EXTENT: f32 = 20.0;

const TITLE_CHROME_RESERVE: f32 = 38.0;
const CLOSE_CHROME_RESERVE: f32 = 28.0;
const CLOSE_RIGHT_INSET: f32 = 6.0;
const PROJECT_PATH_WIDTH_RATIO: f32 = 0.22;
const PROJECT_PATH_MIN_WIDTH: f32 = 150.0;
const PROJECT_PATH_MAX_WIDTH: f32 = 260.0;
const NARROW_VISIBLE_TAB_CAP: usize = 2;

pub(crate) fn main_page_tab_preferred_width_from_title_width(title_width: f32) -> f32 {
    main_page_tab_preferred_width_from_title_width_with_close(title_width, false)
}

pub(crate) fn main_page_tab_preferred_width_from_title_width_with_close(
    title_width: f32,
    closeable: bool,
) -> f32 {
    let title_width = if title_width.is_finite() {
        title_width.max(0.0)
    } else {
        0.0
    };
    let close_reserve = if closeable { CLOSE_CHROME_RESERVE } else { 0.0 };

    (title_width + TITLE_CHROME_RESERVE + close_reserve)
        .clamp(MAIN_PAGE_TAB_MIN_WIDTH, MAIN_PAGE_TAB_MAX_WIDTH)
}

pub(crate) fn main_page_tab_close_frame(tab: UiFrame) -> UiFrame {
    let extent = MAIN_PAGE_TAB_CLOSE_EXTENT
        .min(tab.width.max(0.0))
        .min(tab.height.max(0.0));
    UiFrame::new(
        (tab.x + tab.width - CLOSE_RIGHT_INSET - extent).max(tab.x),
        tab.y + ((tab.height - extent) * 0.5).max(0.0),
        extent,
        extent,
    )
}

pub(crate) fn main_page_project_path_width(shell_width: f32) -> f32 {
    let shell_width = if shell_width.is_finite() {
        shell_width.max(0.0)
    } else {
        0.0
    };
    let primary_chrome_reserve = MAIN_PAGE_TAB_CHROME_SIDE_INSET * 2.0
        + MAIN_PAGE_TAB_MIN_WIDTH
        + MAIN_PAGE_TAB_GAP * 2.0
        + MAIN_PAGE_TAB_OVERFLOW_WIDTH;
    let available_width = (shell_width - primary_chrome_reserve).max(0.0);
    if available_width < PROJECT_PATH_MIN_WIDTH {
        return 0.0;
    }

    shell_width
        .mul_add(PROJECT_PATH_WIDTH_RATIO, 0.0)
        .clamp(PROJECT_PATH_MIN_WIDTH, PROJECT_PATH_MAX_WIDTH)
        .min(available_width)
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
}
