use super::super::*;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::ui_perf::{UiPerfCounter, record_current_ui_perf_counter};

impl RetainedEditorHost {
    pub(super) fn sync_recompute_viewport_and_pointer_layouts(
        &mut self,
        model: &mut WorkbenchViewModel,
        chrome: &mut crate::ui::workbench::snapshot::EditorChromeSnapshot,
        componentized_workbench_layout_frames: callback_dispatch::BuiltinWorkbenchWindowLayoutFrames,
        floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    ) {
        let viewport_content_frame = componentized_workbench_layout_frames
            .viewport_content_frame
            .filter(super::shell_metrics::ui_frame_is_visible)
            .unwrap_or_default();

        if let Some(next_viewport_size) = viewport_size_from_frame(viewport_content_frame) {
            if next_viewport_size != self.viewport_size {
                zircon_runtime::profile_scope!(
                    "editor",
                    "retained_host",
                    "recompute_viewport_resize"
                );
                self.viewport_size = next_viewport_size;
                self.apply_viewport_resize_effects_in_active_recompute(
                    callback_dispatch::dispatch_viewport_event(
                        &self.runtime,
                        EditorViewportEvent::Resized {
                            width: self.viewport_size.x,
                            height: self.viewport_size.y,
                        },
                    ),
                );
                *chrome = self.build_chrome();
                record_current_ui_perf_counter(UiPerfCounter::WorkbenchModelBuildCount, 1.0);
                let context = self.runtime.project_command_eval_snapshot(chrome);
                let commands = self.runtime.commands().lock();
                *model = WorkbenchViewModel::build_with_context(&commands, chrome, &context);
            }
        }

        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "recompute_pointer_bridge_layouts"
        );
        self.viewport_pointer_bridge
            .update_viewport_frame(UiFrame::new(
                0.0,
                0.0,
                viewport_content_frame.width.max(0.0),
                viewport_content_frame.height.max(0.0),
            ));
        self.shell_pointer_bridge
            .update_layout_with_workbench_layout_frames(
                self.shell_size,
                model.drawer_ring.visible,
                &model.floating_windows,
                componentized_workbench_layout_frames,
                Some(floating_window_projection_bundle),
            );
        self.sync_activity_rail_pointer_layout(model);
        self.sync_host_page_pointer_layout(model);
        self.sync_document_tab_pointer_layout(model, floating_window_projection_bundle);
        self.sync_drawer_header_pointer_layout_with_workbench_layout_frames(
            model,
            componentized_workbench_layout_frames,
        );
    }
}
