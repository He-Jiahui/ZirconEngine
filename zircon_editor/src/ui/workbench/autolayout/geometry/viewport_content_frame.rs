use crate::snapshot::ViewContentKind;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::super::active_tab::active_document_tab;
use super::super::{ShellFrame, WorkbenchChromeMetrics};

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
