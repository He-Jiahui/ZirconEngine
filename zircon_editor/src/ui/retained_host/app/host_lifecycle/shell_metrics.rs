use super::super::*;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(in crate::ui::retained_host::app::host_lifecycle) fn ui_frame_is_visible(
    frame: &UiFrame,
) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn build_chrome(
        &self,
    ) -> crate::ui::workbench::snapshot::EditorChromeSnapshot {
        record_current_ui_perf_counter(UiPerfCounter::ChromeSnapshotCount, 1.0);
        self.runtime.chrome_snapshot()
    }

    pub(in crate::ui::retained_host::app) fn sync_shell_size(&mut self) {
        let bootstrap = self.ui.get_host_window_bootstrap();
        let next = ShellSizePx::new(
            bootstrap.shell_frame.width.max(1.0),
            bootstrap.shell_frame.height.max(1.0),
        );
        if (next.width - self.shell_size.width).abs() <= 0.5
            && (next.height - self.shell_size.height).abs() <= 0.5
        {
            return;
        }
        self.shell_size = next;
        self.invalidate_host(
            HostInvalidationMask::WINDOW_METRICS.union(HostInvalidationMask::PRESENTATION_DATA),
        );
    }

    pub(in crate::ui::retained_host::app) fn publish_refresh_invalidation_diagnostics(&self) {
        self.ui
            .set_host_refresh_invalidation_diagnostics(self.invalidation.diagnostics_snapshot());
    }
}
