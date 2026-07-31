use super::super::*;
use crate::ui::retained_host::ui_perf::{UiPerfCounter, record_current_ui_perf_counter};
use zircon_runtime::diagnostic_log::{
    DiagnosticLogLevel, diagnostic_log_allows, write_diagnostic_log,
};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute::invalidation) fn record_slow_path_recompute(
        &mut self,
        recompute_reasons: &HostInvalidationMask,
    ) {
        let slow_path_rebuild = self.invalidation.record_slow_path_rebuild();
        record_current_ui_perf_counter(UiPerfCounter::SlowPathRebuildCount, 1.0);
        record_current_ui_perf_counter(UiPerfCounter::ChromeCommandFullRebuildCount, 1.0);
        self.publish_refresh_invalidation_diagnostics();
        if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
            write_diagnostic_log(
                "editor_host_invalidation",
                format!(
                    "slow_path count={} reasons={} legacy_dirty_flags={{layout:{},presentation:{},window_metrics:{},render:{}}} {}",
                    slow_path_rebuild,
                    recompute_reasons.summary(),
                    self.layout_dirty,
                    self.presentation_dirty,
                    self.window_metrics_dirty,
                    self.render_dirty,
                    self.invalidation.stats_summary()
                ),
            );
        }
    }
}
