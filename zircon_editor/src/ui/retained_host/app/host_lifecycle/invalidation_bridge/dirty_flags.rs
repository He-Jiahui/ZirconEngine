use crate::ui::retained_host::app::{HostInvalidationMask, RetainedEditorHost};
use crate::ui::retained_host::ui_perf::{
    UiPerfCounter, UiPerfScenario, current_ui_perf_scenario, record_current_ui_perf_counter,
};

fn record_ui_dirty_mask(mask: HostInvalidationMask) {
    if mask.requires_layout() {
        record_current_ui_perf_counter(UiPerfCounter::DirtyLayout, 1.0);
    }
    if mask.requires_presentation() {
        record_current_ui_perf_counter(UiPerfCounter::DirtyPresentation, 1.0);
    }
    if mask.requires_render() {
        record_current_ui_perf_counter(UiPerfCounter::DirtyRender, 1.0);
    }
    if mask.intersects(
        HostInvalidationMask::PAINT_ONLY
            .union(HostInvalidationMask::POINTER_HOVER)
            .union(HostInvalidationMask::VIEWPORT_IMAGE),
    ) {
        record_current_ui_perf_counter(UiPerfCounter::DirtyPaintOnly, 1.0);
    }
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn invalidate_host(
        &mut self,
        mask: HostInvalidationMask,
    ) {
        self.capture_pending_ui_perf_scenario();
        record_ui_dirty_mask(mask);
        self.invalidation.invalidate(mask);
        if mask.requires_window_metrics() {
            self.window_metrics_dirty = true;
        }
        if mask.requires_layout() {
            self.layout_dirty = true;
        }
        if mask.requires_presentation() || mask.requires_hit_test() {
            self.presentation_dirty = true;
        }
        if mask.requires_render() {
            self.render_dirty = true;
        }
    }

    pub(in crate::ui::retained_host::app) fn record_paint_only_invalidation(
        &mut self,
        mask: HostInvalidationMask,
    ) {
        let mask = mask.union(HostInvalidationMask::PAINT_ONLY);
        self.capture_pending_ui_perf_scenario();
        record_ui_dirty_mask(mask);
        self.invalidation.invalidate(mask);
        self.publish_refresh_invalidation_diagnostics();
    }

    fn capture_pending_ui_perf_scenario(&mut self) {
        let scenario = current_ui_perf_scenario();
        if scenario != UiPerfScenario::Startup {
            self.pending_ui_perf_scenario = Some(scenario);
        }
    }
}
