//! 文档页签绘制与拖拽命中共享的度量；标题宽度来自排版测量，关闭区预留与设计令牌共同决定尺寸。
use zircon_runtime_interface::ui::design_tokens::EditorTypographyTokens;

pub(crate) const DOCUMENT_TAB_STRIP_X: f32 = 8.0;
pub(crate) const DOCUMENT_TAB_STRIP_Y: f32 = 1.0;
pub(crate) const DOCUMENT_TAB_MIN_WIDTH: f32 = 124.0;
pub(crate) const DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH: f32 = 156.0;
pub(crate) const DOCUMENT_TAB_MAX_WIDTH: f32 = 220.0;
pub(crate) const DOCUMENT_TAB_HEIGHT: f32 = 30.0;
pub(crate) const DOCUMENT_TAB_GAP: f32 = 4.0;
pub(crate) const DOCUMENT_TAB_CLOSE_EXTENT: f32 = 20.0;
pub(crate) const DOCUMENT_TAB_CLOSE_RIGHT_INSET: f32 = 8.0;
pub(crate) const DOCUMENT_TAB_CLOSE_TOP_INSET: f32 = 6.0;
pub(crate) const DOCUMENT_TAB_TITLE_FONT_SIZE: f32 = EditorTypographyTokens::WORKBENCH_BODY_SIZE;

const TITLE_CHROME_RESERVE: f32 = 42.0;
const CLOSEABLE_TITLE_CHROME_RESERVE: f32 = 70.0;

// 根据已测量标题申请有界页签宽度，绘制与拖拽必须共用此契约；不是按字符数估算。
pub(crate) fn document_tab_preferred_width_from_title_width(
    title_width: f32,
    closeable: bool,
) -> f32 {
    let reserve = if closeable {
        CLOSEABLE_TITLE_CHROME_RESERVE
    } else {
        TITLE_CHROME_RESERVE
    };
    let minimum = if closeable {
        DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH
    } else {
        DOCUMENT_TAB_MIN_WIDTH
    };
    let title_width = if title_width.is_finite() {
        title_width.max(0.0)
    } else {
        0.0
    };

    (title_width + reserve).clamp(minimum, DOCUMENT_TAB_MAX_WIDTH)
}

// 供已解析页签的关闭命中区定位；调用方须传入与绘制相同的合法页签坐标和宽度。
pub(crate) fn document_tab_close_x(tab_x: f32, tab_width: f32) -> f32 {
    tab_x + tab_width - DOCUMENT_TAB_CLOSE_RIGHT_INSET - DOCUMENT_TAB_CLOSE_EXTENT
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;
