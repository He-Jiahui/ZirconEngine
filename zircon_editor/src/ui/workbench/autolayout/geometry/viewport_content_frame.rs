use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::ViewContentKind;

use super::super::active_tab::active_document_tab;
use super::super::{ShellFrame, WorkbenchChromeMetrics};

/// 供viewport绘制和命中的logical内容区；文档标题与Scene/Game工具条不属于渲染视口。
pub(super) fn build_viewport_content_frame(
    model: &WorkbenchViewModel,
    document_frame: ShellFrame,
    metrics: &WorkbenchChromeMetrics,
) -> ShellFrame {
    let viewport_toolbar_height = active_document_tab(model)
        .map(|tab| {
            matches!(
                tab.content_kind,
                ViewContentKind::Scene | ViewContentKind::Game
            )
        })
        .unwrap_or(false)
        .then_some(metrics.viewport_toolbar_height)
        .unwrap_or(0.0);

    ShellFrame::new(
        document_frame.x,
        document_frame.y
            + metrics.document_header_height
            + metrics.separator_thickness
            + viewport_toolbar_height,
        document_frame.width,
        (document_frame.height
            - metrics.document_header_height
            - metrics.separator_thickness
            - viewport_toolbar_height)
            .max(0.0),
    )
}
