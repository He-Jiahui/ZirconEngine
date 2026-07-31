use super::super::*;
use crate::ui::retained_host::ui_perf::{UiPerfCounter, record_current_ui_perf_counter};
use zircon_runtime::diagnostic_log::{
    DiagnosticLogLevel, diagnostic_log_allows, write_diagnostic_log,
};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute::invalidation) fn complete_paint_only_recompute(
        &mut self,
        recompute_reasons: &HostInvalidationMask,
    ) {
        record_current_ui_perf_counter(UiPerfCounter::ChromeCommandPatchCount, 1.0);
        self.presentation_dirty = false;
        self.layout_dirty = false;
        self.window_metrics_dirty = false;
        self.publish_refresh_invalidation_diagnostics();
        if diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
            write_diagnostic_log(
                "editor_host_invalidation",
                format!(
                    "paint_only_fast_path reasons={} legacy_dirty_flags={{layout:{},presentation:{},window_metrics:{},render:{}}} {}",
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
