use super::*;
use crate::ui::retained_host::floating_window_projection::{
    build_floating_window_projection_bundle_with_shared_source,
    resolve_floating_window_projection_base_outer_frame,
    resolve_floating_window_projection_shared_source, FloatingWindowProjectionBundle,
};

impl RetainedEditorHost {
    pub(super) fn build_recompute_floating_window_projection_bundle(
        &mut self,
        model: &WorkbenchViewModel,
    ) -> FloatingWindowProjectionBundle {
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_floating_source_bridge"
            );
            let _ = self
                .floating_window_source_bridge
                .recompute_layout(UiSize::new(self.shell_size.width, self.shell_size.height));
        }
        let floating_window_shared_source = resolve_floating_window_projection_shared_source(
            &self.floating_window_source_bridge.source_frames(),
        );
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_sync_floating_bounds"
            );
            for (window_index, window) in model.floating_windows.iter().enumerate() {
                let frame = resolve_floating_window_projection_base_outer_frame(
                    window,
                    window_index,
                    floating_window_shared_source,
                );
                self.editor_manager.sync_native_window_projection_bounds(
                    &window.window_id,
                    [frame.x, frame.y, frame.width, frame.height],
                );
            }
        }
        let native_window_hosts = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_native_window_hosts"
            );
            self.editor_manager.native_window_hosts()
        };
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_floating_projection_bundle"
            );
            build_floating_window_projection_bundle_with_shared_source(
                model,
                floating_window_shared_source,
                &self.chrome_metrics,
                &native_window_hosts,
            )
        }
    }
}
