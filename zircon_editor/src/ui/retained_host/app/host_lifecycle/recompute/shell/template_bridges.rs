use super::super::*;
use crate::ui::retained_host::callback_dispatch;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime::diagnostic_log::write_error;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute::shell) fn recompute_shell_template_bridge_layout_frames(
        &mut self,
        model: &WorkbenchViewModel,
    ) -> callback_dispatch::BuiltinWorkbenchWindowLayoutFrames {
        let shell_size = UiSize::new(self.shell_size.width, self.shell_size.height);
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_root_template_bridge"
            );
            if let Err(error) = self.template_bridge.recompute_layout_with_workbench_model(
                shell_size,
                model,
                &self.chrome_metrics,
            ) {
                write_error(
                    "editor_root_template_bridge_layout",
                    format!("Root template bridge layout recompute failed: {error}"),
                );
            }
        }
        let workbench_mount_frame = self
            .template_bridge
            .root_shell_frames()
            .componentized_workbench_mount_frame(shell_size);
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_workbench_window_bridge"
            );
            if let Err(error) = self
                .workbench_window_bridge
                .recompute_mounted_layout_with_workbench_model_at_scale(
                    workbench_mount_frame,
                    self.shell_scale_factor,
                    model,
                    &self.chrome_metrics,
                )
            {
                write_error(
                    "editor_workbench_template_bridge_layout",
                    format!("Workbench template bridge layout recompute failed: {error}"),
                );
            }
        }
        self.workbench_window_bridge.layout_frames()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn template_bridge_recompute_failures_are_not_silently_discarded() {
        let source = include_str!("template_bridges.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("test module should remain isolated from production bridge recompute code");

        assert!(!production
            .contains("let _ = self.template_bridge.recompute_layout_with_workbench_model"));
        assert!(!production.contains("let _ = self\n                .workbench_window_bridge"));
        assert!(production.contains("editor_root_template_bridge_layout"));
        assert!(production.contains("editor_workbench_template_bridge_layout"));
        assert!(production.matches("write_error(").count() >= 2);
    }
}
