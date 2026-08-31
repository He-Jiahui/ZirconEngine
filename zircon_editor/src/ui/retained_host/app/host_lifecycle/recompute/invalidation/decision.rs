use super::super::*;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use crate::ui::retained_host::HostShellContentScope;
use crate::ui::workbench::view::ViewInstanceId;

pub(in crate::ui::retained_host::app::host_lifecycle::recompute) struct RecomputeInvalidationDecision
{
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) paint_only_reasons:
        HostInvalidationMask,
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) reuse_shell_layout: bool,
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) target:
        RecomputeInvalidationTarget,
}

pub(in crate::ui::retained_host::app::host_lifecycle::recompute) enum RecomputeInvalidationTarget {
    Full,
    WindowMetrics,
    ShellContent(HostShellContentScope),
    ViewPresentation(Vec<ViewInstanceId>),
    WorkbenchProjection,
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) fn begin_recompute_invalidation_phase(
        &mut self,
    ) -> Option<RecomputeInvalidationDecision> {
        let pending_transaction = self.invalidation.take_recompute_transaction();
        let pending_reasons = pending_transaction.reasons();
        let scope_count = pending_transaction.scope_count();
        let legacy_dirty_reasons = HostInvalidationMask::from_dirty_flags(
            self.layout_dirty,
            self.presentation_dirty,
            self.window_metrics_dirty,
            self.render_dirty,
        );
        record_current_ui_perf_counter(UiPerfCounter::HostInvalidationTransactionCount, 1.0);
        record_current_ui_perf_counter(
            UiPerfCounter::HostInvalidationScopeCount,
            scope_count as f64,
        );
        if !legacy_dirty_reasons.is_empty() {
            record_current_ui_perf_counter(
                UiPerfCounter::HostInvalidationLegacyDirtyTransactionCount,
                1.0,
            );
        }
        let reuse_shell_layout = shell_content_reuses_committed_layout(pending_reasons)
            && !self.layout_dirty
            && !self.window_metrics_dirty;
        let recompute_reasons = pending_reasons.union(legacy_dirty_reasons);
        if let Some(scope) = pending_transaction.shell_content_scope().filter(|_| {
            reuse_shell_layout && legacy_dirty_allows_shell_content_patch(legacy_dirty_reasons)
        }) {
            record_current_ui_perf_counter(
                UiPerfCounter::HostInvalidationShellContentTargetCount,
                1.0,
            );
            return Some(RecomputeInvalidationDecision {
                paint_only_reasons: HostInvalidationMask::NONE,
                reuse_shell_layout: true,
                target: RecomputeInvalidationTarget::ShellContent(scope),
            });
        }
        if workbench_projection_reuses_host(pending_reasons, legacy_dirty_reasons) {
            record_current_ui_perf_counter(
                UiPerfCounter::HostInvalidationWorkbenchProjectionTargetCount,
                1.0,
            );
            return Some(RecomputeInvalidationDecision {
                paint_only_reasons: pending_reasons.intersection(HostInvalidationMask::PAINT_ONLY),
                reuse_shell_layout: false,
                target: RecomputeInvalidationTarget::WorkbenchProjection,
            });
        }
        if legacy_dirty_reasons.is_empty() {
            if let Some(view_ids) = pending_transaction.presentation_only_view_ids() {
                record_current_ui_perf_counter(
                    UiPerfCounter::HostInvalidationViewPresentationTargetCount,
                    1.0,
                );
                return Some(RecomputeInvalidationDecision {
                    paint_only_reasons: HostInvalidationMask::NONE,
                    reuse_shell_layout: false,
                    target: RecomputeInvalidationTarget::ViewPresentation(view_ids),
                });
            }
        }
        if window_metrics_reuses_committed_shell(pending_reasons, legacy_dirty_reasons) {
            record_current_ui_perf_counter(
                UiPerfCounter::HostInvalidationWindowMetricsTargetCount,
                1.0,
            );
            return Some(RecomputeInvalidationDecision {
                paint_only_reasons: HostInvalidationMask::NONE,
                reuse_shell_layout: false,
                target: RecomputeInvalidationTarget::WindowMetrics,
            });
        }
        let paint_only_reasons = recompute_reasons.intersection(
            HostInvalidationMask::PAINT_ONLY
                .union(HostInvalidationMask::POINTER_HOVER)
                .union(HostInvalidationMask::VIEWPORT_IMAGE),
        );
        let pure_paint_only = !paint_only_reasons.is_empty()
            && !recompute_reasons.requires_layout()
            && !recompute_reasons.requires_presentation()
            && !recompute_reasons.requires_window_metrics()
            && !recompute_reasons.requires_hit_test()
            && !recompute_reasons.requires_render();
        if pure_paint_only {
            record_current_ui_perf_counter(
                UiPerfCounter::HostInvalidationPaintOnlyTargetCount,
                1.0,
            );
            self.complete_paint_only_recompute(&recompute_reasons);
            return None;
        }

        record_current_ui_perf_counter(UiPerfCounter::HostInvalidationFullTargetCount, 1.0);
        self.record_slow_path_recompute(&recompute_reasons, scope_count);
        Some(RecomputeInvalidationDecision {
            paint_only_reasons,
            reuse_shell_layout,
            target: RecomputeInvalidationTarget::Full,
        })
    }
}

fn workbench_projection_reuses_host(
    pending_reasons: HostInvalidationMask,
    legacy_dirty_reasons: HostInvalidationMask,
) -> bool {
    let allowed_pending = HostInvalidationMask::WORKBENCH_PROJECTION
        .union(HostInvalidationMask::PAINT_ONLY)
        .union(HostInvalidationMask::RENDER);
    pending_reasons.contains(HostInvalidationMask::WORKBENCH_PROJECTION)
        && pending_reasons.intersection(allowed_pending) == pending_reasons
        && legacy_dirty_reasons.intersection(HostInvalidationMask::RENDER) == legacy_dirty_reasons
}

fn shell_content_reuses_committed_layout(pending_reasons: HostInvalidationMask) -> bool {
    let allowed_reasons =
        HostInvalidationMask::SHELL_CONTENT.union(HostInvalidationMask::PRESENTATION_DATA);
    pending_reasons.contains(HostInvalidationMask::SHELL_CONTENT)
        && pending_reasons.intersection(allowed_reasons) == pending_reasons
}

fn legacy_dirty_allows_shell_content_patch(legacy_dirty_reasons: HostInvalidationMask) -> bool {
    legacy_dirty_reasons.intersection(HostInvalidationMask::PRESENTATION_DATA)
        == legacy_dirty_reasons
}

fn window_metrics_reuses_committed_shell(
    pending_reasons: HostInvalidationMask,
    legacy_dirty_reasons: HostInvalidationMask,
) -> bool {
    let allowed_legacy =
        HostInvalidationMask::WINDOW_METRICS.union(HostInvalidationMask::PRESENTATION_DATA);
    pending_reasons == HostInvalidationMask::WINDOW_METRICS
        && legacy_dirty_reasons.intersection(allowed_legacy) == legacy_dirty_reasons
}

#[cfg(test)]
mod tests {
    use super::{
        legacy_dirty_allows_shell_content_patch, shell_content_reuses_committed_layout,
        window_metrics_reuses_committed_shell, workbench_projection_reuses_host,
    };
    use crate::ui::retained_host::HostInvalidationMask;

    #[test]
    fn shell_content_accepts_the_legacy_presentation_compatibility_bit() {
        assert!(shell_content_reuses_committed_layout(
            HostInvalidationMask::SHELL_CONTENT
        ));
        assert!(shell_content_reuses_committed_layout(
            HostInvalidationMask::SHELL_CONTENT.union(HostInvalidationMask::PRESENTATION_DATA)
        ));
    }

    #[test]
    fn workbench_projection_accepts_render_and_paint_without_full_shell_recompute() {
        assert!(workbench_projection_reuses_host(
            HostInvalidationMask::WORKBENCH_PROJECTION
                .union(HostInvalidationMask::PAINT_ONLY)
                .union(HostInvalidationMask::RENDER),
            HostInvalidationMask::RENDER,
        ));
    }

    #[test]
    fn workbench_projection_rejects_global_presentation() {
        assert!(!workbench_projection_reuses_host(
            HostInvalidationMask::WORKBENCH_PROJECTION
                .union(HostInvalidationMask::PRESENTATION_DATA),
            HostInvalidationMask::PRESENTATION_DATA,
        ));
    }

    #[test]
    fn workbench_projection_rejects_layout_hit_test_and_window_metrics() {
        for incompatible in [
            HostInvalidationMask::LAYOUT,
            HostInvalidationMask::HIT_TEST,
            HostInvalidationMask::WINDOW_METRICS,
        ] {
            assert!(!workbench_projection_reuses_host(
                HostInvalidationMask::WORKBENCH_PROJECTION.union(incompatible),
                incompatible,
            ));
        }
    }

    #[test]
    fn shell_content_rejects_coalesced_layout_or_render_work() {
        assert!(!shell_content_reuses_committed_layout(
            HostInvalidationMask::SHELL_CONTENT.union(HostInvalidationMask::LAYOUT)
        ));
        assert!(!shell_content_reuses_committed_layout(
            HostInvalidationMask::SHELL_CONTENT.union(HostInvalidationMask::RENDER)
        ));
    }

    #[test]
    fn shell_content_accepts_only_the_legacy_presentation_mirror() {
        assert!(legacy_dirty_allows_shell_content_patch(
            HostInvalidationMask::NONE
        ));
        assert!(legacy_dirty_allows_shell_content_patch(
            HostInvalidationMask::PRESENTATION_DATA
        ));
        assert!(!legacy_dirty_allows_shell_content_patch(
            HostInvalidationMask::PRESENTATION_DATA.union(HostInvalidationMask::RENDER)
        ));
    }

    #[test]
    fn pure_window_metrics_reuses_the_committed_shell_stage() {
        assert!(window_metrics_reuses_committed_shell(
            HostInvalidationMask::WINDOW_METRICS,
            HostInvalidationMask::WINDOW_METRICS.union(HostInvalidationMask::PRESENTATION_DATA),
        ));
    }

    #[test]
    fn window_metrics_rejects_coalesced_business_presentation() {
        assert!(!window_metrics_reuses_committed_shell(
            HostInvalidationMask::WINDOW_METRICS.union(HostInvalidationMask::PRESENTATION_DATA),
            HostInvalidationMask::WINDOW_METRICS.union(HostInvalidationMask::PRESENTATION_DATA),
        ));
        assert!(!window_metrics_reuses_committed_shell(
            HostInvalidationMask::WINDOW_METRICS,
            HostInvalidationMask::WINDOW_METRICS.union(HostInvalidationMask::LAYOUT),
        ));
    }
}
