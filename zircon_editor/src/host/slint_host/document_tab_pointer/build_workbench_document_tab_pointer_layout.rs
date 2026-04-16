use zircon_ui::UiFrame;

use crate::{WorkbenchChromeMetrics, WorkbenchShellGeometry, WorkbenchViewModel};

use super::document_region_frame_ext::DocumentRegionFrameExt;
use super::workbench_document_tab_pointer_item::WorkbenchDocumentTabPointerItem;
use super::workbench_document_tab_pointer_layout::WorkbenchDocumentTabPointerLayout;
use super::workbench_document_tab_pointer_surface::WorkbenchDocumentTabPointerSurface;

pub(crate) fn build_workbench_document_tab_pointer_layout(
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
) -> WorkbenchDocumentTabPointerLayout {
    let mut surfaces = Vec::new();
    if !model.document_tabs.is_empty() {
        let document_region = geometry.document_region_frame();
        surfaces.push(WorkbenchDocumentTabPointerSurface {
            key: "main".to_string(),
            strip_frame: UiFrame::new(
                document_region.x,
                document_region.y,
                document_region.width,
                metrics.document_header_height,
            ),
            items: model
                .document_tabs
                .iter()
                .map(|tab| WorkbenchDocumentTabPointerItem {
                    instance_id: tab.instance_id.0.clone(),
                    closeable: tab.closeable,
                })
                .collect(),
        });
    }

    surfaces.extend(model.floating_windows.iter().map(|window| {
        let frame = geometry.floating_window_frame(&window.window_id);
        WorkbenchDocumentTabPointerSurface {
            key: window.window_id.0.clone(),
            strip_frame: UiFrame::new(
                frame.x,
                frame.y,
                frame.width,
                metrics.document_header_height,
            ),
            items: window
                .tabs
                .iter()
                .map(|tab| WorkbenchDocumentTabPointerItem {
                    instance_id: tab.instance_id.0.clone(),
                    closeable: tab.closeable,
                })
                .collect(),
        }
    }));

    WorkbenchDocumentTabPointerLayout { surfaces }
}
