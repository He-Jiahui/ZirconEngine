use super::super::*;
use crate::core::logging::{EditorLogService, LogEntry, LogSeverity, LogSource};
use crate::ui::retained_host::callback_dispatch;
use crate::ui::workbench::autolayout::ResolutionContext;
use crate::ui::workbench::model::WorkbenchViewModel;

impl RetainedEditorHost {
    // 根模板先确定工作台挂载框，再以同一有效缩放重排已挂载桥；错误进入有界日志供完整重算诊断。
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute::shell) fn recompute_shell_template_bridge_layout_frames(
        &mut self,
        model: &WorkbenchViewModel,
    ) -> callback_dispatch::BuiltinWorkbenchWindowLayoutFrames {
        let shell_size = UiSize::new(self.shell_size.width, self.shell_size.height);
        let resolution = ResolutionContext::from_physical_size_with_scale_mode(
            self.shell_size,
            self.shell_scale_factor,
            self.shell_scale_mode,
        );
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "recompute_root_template_bridge"
            );
            if let Err(error) = self
                .template_bridge
                .recompute_layout_with_workbench_model_at_scale(
                    shell_size,
                    resolution.effective_scale_factor(),
                    model,
                    &self.chrome_metrics,
                )
            {
                emit_template_bridge_layout_error(
                    self.runtime.context().logs(),
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
                    resolution.effective_scale_factor(),
                    model,
                    &self.chrome_metrics,
                )
            {
                emit_template_bridge_layout_error(
                    self.runtime.context().logs(),
                    "editor_workbench_template_bridge_layout",
                    format!("Workbench template bridge layout recompute failed: {error}"),
                );
            }
        }
        self.workbench_window_bridge.layout_frames()
    }
}

pub(super) fn emit_template_bridge_layout_error(
    logs: &EditorLogService,
    component: &str,
    error: impl std::fmt::Display,
) {
    let entry = LogEntry::new(
        LogSource::editor(),
        LogSeverity::Error,
        format!("{component} {error}"),
        0,
        None,
    )
    .or_else(|_| {
        LogEntry::new(
            LogSource::editor(),
            LogSeverity::Error,
            "editor_template_bridge layout diagnostic exceeds the log-entry limit.",
            0,
            None,
        )
    });
    if let Ok(entry) = entry {
        let _ = logs.emit(entry);
    }
}

#[cfg(test)]
#[path = "tests/template_bridges.rs"]
mod tests;
