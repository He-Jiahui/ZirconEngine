use super::super::*;
use crate::ui::retained_host::callback_dispatch;
use crate::ui::workbench::model::WorkbenchViewModel;

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
            let _ = self.template_bridge.recompute_layout_with_workbench_model(
                shell_size,
                model,
                &self.chrome_metrics,
            );
        }
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_workbench_window_bridge"
            );
            let _ = self
                .workbench_window_bridge
                .recompute_layout_with_workbench_model_at_scale(
                    shell_size,
                    self.shell_scale_factor,
                    model,
                    &self.chrome_metrics,
                );
        }
        self.workbench_window_bridge.layout_frames()
    }
}
