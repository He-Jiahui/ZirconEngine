use super::super::*;

pub(in crate::ui::retained_host::app::host_lifecycle::recompute) struct RecomputeInvalidationDecision
{
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) paint_only_reasons:
        HostInvalidationMask,
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::host_lifecycle::recompute) fn begin_recompute_invalidation_phase(
        &mut self,
    ) -> Option<RecomputeInvalidationDecision> {
        let pending_reasons = self.invalidation.take_recompute_reasons();
        let recompute_reasons = if pending_reasons.is_empty() {
            HostInvalidationMask::from_dirty_flags(
                self.layout_dirty,
                self.presentation_dirty,
                self.window_metrics_dirty,
                self.render_dirty,
            )
        } else {
            pending_reasons
        };
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
            self.complete_paint_only_recompute(&recompute_reasons);
            return None;
        }

        self.record_slow_path_recompute(&recompute_reasons);
        Some(RecomputeInvalidationDecision { paint_only_reasons })
    }
}
