use crate::ui::workbench::snapshot::ViewContentKind;
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::super::super::super::RetainedEditorHost;
use super::super::frame::ui_frame_size;
use super::super::workbench_regions::{
    active_drawer_region_for_kind, active_main_page_matches_kind,
    active_workbench_main_page_matches_kind,
};

pub(super) fn resolve_workbench_host_frame_backed_size_for_kind(
    host: &RetainedEditorHost,
    kind: ViewContentKind,
) -> Option<UiSize> {
    let workbench_layout_frames = host.workbench_window_bridge.layout_frames();
    let chrome = host.runtime.chrome_snapshot();
    let workbench = &chrome.workbench;
    if let Some(region) = active_drawer_region_for_kind(workbench, kind) {
        return workbench_layout_frames
            .drawer_content_frame(region)
            .and_then(ui_frame_size);
    }

    if active_workbench_main_page_matches_kind(workbench, kind) {
        if matches!(kind, ViewContentKind::Scene | ViewContentKind::Game) {
            if let Some(size) = workbench_layout_frames
                .viewport_content_frame
                .and_then(ui_frame_size)
            {
                return Some(size);
            }
        }
        if let Some(size) = workbench_layout_frames
            .document_region_frame
            .and_then(ui_frame_size)
        {
            return Some(size);
        }
        return None;
    }

    if active_main_page_matches_kind(workbench, kind) {
        let root_shell_frames = host.template_bridge.root_shell_frames();
        return root_shell_frames
            .pane_surface_frame
            .and_then(ui_frame_size)
            .or_else(|| {
                root_shell_frames
                    .document_host_frame
                    .and_then(ui_frame_size)
            });
    }

    None
}
