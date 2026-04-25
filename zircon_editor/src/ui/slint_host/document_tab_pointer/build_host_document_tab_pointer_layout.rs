use zircon_runtime::ui::layout::UiFrame;

use crate::ui::slint_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::slint_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::slint_host::root_shell_projection::resolve_root_document_tabs_frame;
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::autolayout::WorkbenchShellGeometry;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::host_document_tab_pointer_item::HostDocumentTabPointerItem;
use super::host_document_tab_pointer_layout::HostDocumentTabPointerLayout;
use super::host_document_tab_pointer_surface::HostDocumentTabPointerSurface;

pub(crate) fn build_host_document_tab_pointer_layout(
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) -> HostDocumentTabPointerLayout {
    let mut surfaces = Vec::new();
    if !model.document_tabs.is_empty() {
        let document_tabs = resolve_root_document_tabs_frame(geometry, metrics, shared_root_frames);
        surfaces.push(HostDocumentTabPointerSurface {
            key: "main".to_string(),
            strip_frame: UiFrame::new(
                document_tabs.x,
                document_tabs.y,
                document_tabs.width,
                document_tabs.height,
            ),
            items: model
                .document_tabs
                .iter()
                .map(|tab| HostDocumentTabPointerItem {
                    instance_id: tab.instance_id.0.clone(),
                    closeable: tab.closeable,
                })
                .collect(),
        });
    }

    surfaces.extend(model.floating_windows.iter().map(|window| {
        let frame = floating_window_projection_bundle
            .tab_strip_frame(&window.window_id)
            .unwrap_or_default();
        HostDocumentTabPointerSurface {
            key: window.window_id.0.clone(),
            strip_frame: UiFrame::new(frame.x, frame.y, frame.width, frame.height),
            items: window
                .tabs
                .iter()
                .map(|tab| HostDocumentTabPointerItem {
                    instance_id: tab.instance_id.0.clone(),
                    closeable: tab.closeable,
                })
                .collect(),
        }
    }));

    HostDocumentTabPointerLayout { surfaces }
}
