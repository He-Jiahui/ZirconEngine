//! 主页面页签和次级项目路径的共享度量；逻辑宽度分层先限制数量，宿主再分配实际标题和溢出空间。
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

// 不可关闭主页面沿用同一标题测量契约，供页签投影共享字体与宽度边界。
pub(crate) fn main_page_tab_preferred_width_from_title_width(title_width: f32) -> f32 {
    main_page_tab_preferred_width_from_title_width_with_close(title_width, false)
}

// 可关闭页面另预留关闭命中区；输入是排版测得的逻辑宽度，显示截断由宿主处理。
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

// 从已解析页签框取得内部关闭命中框；调用方须沿用绘制的同一合法坐标系。
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

// 项目路径是次级chrome信息；预算不足时先让位给主页面与溢出入口。
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

// 按shell逻辑宽度层给出可见页签数量上限；宿主仍须完成宽度分配，并把余项放入溢出列表。
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
#[path = "tests/metrics.rs"]
mod tests;
