use crate::ui::retained_host::app::{HostInvalidationMask, RetainedEditorHost};
use crate::ui::template_runtime::UNREAL_EDITOR_WINDOW_DOCUMENT_ID;
use crate::ui::workbench::snapshot::StatusTaskProgressSnapshot;

impl RetainedEditorHost {
    // 状态先写 Runtime 快照；当前工作台模板可局部提交时只投影该变化，其他页面回退到全局展示刷新。
    pub(in crate::ui::retained_host::app) fn set_status_line(
        &mut self,
        message: impl Into<String>,
    ) {
        let message = message.into();
        if !self.runtime.set_retained_status_line(&message) {
            return;
        }
        let active_template = match self
            .active_activity_window_template_document_is(UNREAL_EDITOR_WINDOW_DOCUMENT_ID)
        {
            Ok(active) => active,
            Err(error) => {
                if let Ok(entry) = crate::core::logging::LogEntry::new(
                    crate::core::logging::LogSource::editor(),
                    crate::core::logging::LogSeverity::Error,
                    error.to_string(),
                    0,
                    None,
                ) {
                    let _ = self.runtime.context().logs().emit(entry);
                }
                return;
            }
        };
        if !active_template {
            self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
            return;
        }
        let patched = self
            .workbench_window_bridge
            .prepare_status_line(&message)
            .and_then(|()| self.workbench_window_bridge.refresh_prepared_state_change())
            .is_ok();
        self.invalidate_host(if patched {
            HostInvalidationMask::WORKBENCH_PROJECTION
        } else {
            HostInvalidationMask::PRESENTATION_DATA
        });
    }

    pub(in crate::ui::retained_host::app) fn set_status_task_progress(
        &mut self,
        progress: Option<StatusTaskProgressSnapshot>,
    ) {
        if self.runtime.status_task_progress() == progress {
            return;
        }
        self.runtime.set_status_task_progress(progress);
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }
}
