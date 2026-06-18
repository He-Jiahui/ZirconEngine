use zircon_runtime_interface::ui::layout::UiFrame;

#[cfg(test)]
use crate::ui::retained_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowLayoutFrames;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
#[cfg(test)]
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::host_document_tab_pointer_item::HostDocumentTabPointerItem;
use super::host_document_tab_pointer_layout::HostDocumentTabPointerLayout;
use super::host_document_tab_pointer_surface::HostDocumentTabPointerSurface;

#[cfg(test)]
pub(crate) fn build_host_document_tab_pointer_layout(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) -> HostDocumentTabPointerLayout {
    build_host_document_tab_pointer_layout_with_document_tabs_frame(
        model,
        floating_window_projection_bundle,
        Some(test_root_document_tabs_frame(metrics, shared_root_frames)),
    )
}

pub(crate) fn build_host_document_tab_pointer_layout_with_workbench_layout_frames(
    model: &WorkbenchViewModel,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    workbench_layout_frames: BuiltinWorkbenchWindowLayoutFrames,
) -> HostDocumentTabPointerLayout {
    build_host_document_tab_pointer_layout_with_document_tabs_frame(
        model,
        floating_window_projection_bundle,
        workbench_layout_frames.document_tabs_frame,
    )
}

fn build_host_document_tab_pointer_layout_with_document_tabs_frame(
    model: &WorkbenchViewModel,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    document_tabs_frame: Option<UiFrame>,
) -> HostDocumentTabPointerLayout {
    let mut surfaces = Vec::new();
    if !model.document_tabs.is_empty() {
        let document_tabs = document_tabs_frame
            .filter(ui_frame_is_visible)
            .unwrap_or_default();
        surfaces.push(HostDocumentTabPointerSurface {
            key: "document".to_string(),
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

fn ui_frame_is_visible(frame: &UiFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}

#[cfg(test)]
fn test_root_document_tabs_frame(
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
) -> UiFrame {
    shared_root_frames
        .and_then(|frames| frames.document_tabs_frame)
        .filter(ui_frame_is_visible)
        .or_else(|| {
            shared_root_frames
                .and_then(|frames| frames.document_host_frame)
                .filter(ui_frame_is_visible)
                .map(|document| {
                    UiFrame::new(
                        document.x,
                        document.y,
                        document.width,
                        metrics.document_header_height.max(0.0),
                    )
                })
        })
        .unwrap_or_default()
}
