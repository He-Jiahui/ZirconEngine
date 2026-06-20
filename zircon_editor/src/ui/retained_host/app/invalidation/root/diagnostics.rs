use crate::ui::retained_host::HostInvalidationDiagnostics;

use super::HostInvalidationRoot;

impl HostInvalidationRoot {
    pub(in crate::ui::retained_host::app) fn stats_summary(&self) -> String {
        format!(
            "requests={} layout={} presentation={} render={} paint_only={} hit_test={} window_metrics={} slow_path={} render_path={}",
            self.total_requests,
            self.layout_requests,
            self.presentation_requests,
            self.render_requests,
            self.paint_only_requests,
            self.hit_test_requests,
            self.window_metrics_requests,
            self.slow_path_rebuilds,
            self.render_rebuilds
        )
    }

    pub(in crate::ui::retained_host::app) fn diagnostics_snapshot(
        &self,
    ) -> HostInvalidationDiagnostics {
        HostInvalidationDiagnostics {
            slow_path_rebuild_count: self.slow_path_rebuilds,
            render_rebuild_count: self.render_rebuilds,
            paint_only_request_count: self.paint_only_requests,
        }
    }
}
