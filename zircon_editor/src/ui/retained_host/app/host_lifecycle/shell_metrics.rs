use super::super::*;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

const SHELL_SCALE_FACTOR_EPSILON: f32 = 0.001;

pub(in crate::ui::retained_host::app::host_lifecycle) fn ui_frame_is_visible(
    frame: &UiFrame,
) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}

fn normalized_shell_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
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
        let next_scale_factor = normalized_shell_scale_factor(self.ui.window().scale_factor());
        let size_changed = (next.width - self.shell_size.width).abs() > 0.5
            || (next.height - self.shell_size.height).abs() > 0.5;
        let scale_changed =
            (next_scale_factor - self.shell_scale_factor).abs() > SHELL_SCALE_FACTOR_EPSILON;
        if !size_changed && !scale_changed {
            return;
        }
        let previous_effective_scale = ResolutionContext::from_physical_size_with_scale_mode(
            self.shell_size,
            self.shell_scale_factor,
            self.shell_scale_mode,
        )
        .effective_scale_factor();
        let next_effective_scale = ResolutionContext::from_physical_size_with_scale_mode(
            next,
            next_scale_factor,
            self.shell_scale_mode,
        )
        .effective_scale_factor();
        self.shell_size = next;
        self.shell_scale_factor = next_scale_factor;
        if (next_effective_scale - previous_effective_scale).abs() > SHELL_SCALE_FACTOR_EPSILON {
            apply_host_paint_scale_factor(next_effective_scale);
            self.ui.sync_host_paint_theme();
        }
        self.invalidate_host(HostInvalidationMask::WINDOW_METRICS);
    }

    pub(in crate::ui::retained_host::app) fn publish_refresh_invalidation_diagnostics(&self) {
        self.ui
            .set_host_refresh_invalidation_diagnostics(self.invalidation.diagnostics_snapshot());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn shell_size_sync_reads_committed_event_metrics_without_a_trailing_gate() {
        let source = include_str!("shell_metrics.rs");
        let function = source
            .split("pub(in crate::ui::retained_host::app) fn sync_shell_size")
            .nth(1)
            .and_then(|body| {
                body.split("pub(in crate::ui::retained_host::app) fn publish")
                    .next()
            })
            .expect("sync_shell_size implementation");
        let bootstrap = function
            .find("self.ui.get_host_window_bootstrap()")
            .expect("shell-size sync should read committed window metrics");

        let invalidate = function
            .find("self.invalidate_host(HostInvalidationMask::WINDOW_METRICS)")
            .expect("changed metrics should invalidate the geometry transaction");
        assert!(bootstrap < invalidate);
        assert!(!function.contains("native_resize_reflow_pending"));
    }

    #[test]
    fn shell_metric_sync_projects_the_effective_root_scale_into_the_paint_snapshot() {
        let source = include_str!("shell_metrics.rs");
        let function = source
            .split("pub(in crate::ui::retained_host::app) fn sync_shell_size")
            .nth(1)
            .and_then(|body| {
                body.split("pub(in crate::ui::retained_host::app) fn publish")
                    .next()
            })
            .expect("sync_shell_size implementation");

        assert!(function.contains("ResolutionContext::from_physical_size_with_scale_mode"));
        assert!(function.contains("next_effective_scale"));
        assert!(function.contains("apply_host_paint_scale_factor(next_effective_scale)"));
        assert!(!function.contains("apply_host_paint_scale_factor(next_scale_factor)"));
    }

    #[test]
    fn shell_metric_sync_keeps_the_transaction_resize_specific() {
        let source = include_str!("shell_metrics.rs");
        let function = source
            .split("pub(in crate::ui::retained_host::app) fn sync_shell_size")
            .nth(1)
            .and_then(|body| {
                body.split("pub(in crate::ui::retained_host::app) fn publish")
                    .next()
            })
            .expect("sync_shell_size implementation");

        assert!(function.contains("invalidate_host(HostInvalidationMask::WINDOW_METRICS)"));
        assert!(!function.contains("union(HostInvalidationMask::PRESENTATION_DATA)"));
    }
}
