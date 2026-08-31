use crate::ui::retained_host::app::{HostInvalidationMask, RetainedEditorHost};
use crate::ui::retained_host::ui_perf::{
    current_ui_perf_scenario, record_current_ui_perf_counter, UiPerfCounter, UiPerfScenario,
};
use crate::ui::retained_host::HostShellContentScope;
use crate::ui::workbench::view::ViewInstanceId;

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
        self.invalidate_host_for_scope(None, mask);
    }

    pub(in crate::ui::retained_host::app) fn invalidate_host_for_view(
        &mut self,
        view: &ViewInstanceId,
        mask: HostInvalidationMask,
    ) {
        self.invalidate_host_for_scope(Some(view), mask);
    }

    pub(in crate::ui::retained_host::app) fn invalidate_host_for_shell_content(
        &mut self,
        scope: HostShellContentScope,
        mask: HostInvalidationMask,
    ) {
        self.capture_pending_ui_perf_scenario();
        record_ui_dirty_mask(mask);
        self.invalidation.invalidate_shell_content(scope, mask);
        if mask.requires_window_metrics() {
            self.window_metrics_dirty = true;
        }
        if mask.intersects(HostInvalidationMask::LAYOUT.union(HostInvalidationMask::TREE_STRUCTURE))
        {
            self.layout_dirty = true;
        }
        if mask.requires_presentation() || mask.requires_hit_test() {
            self.presentation_dirty = true;
        }
        if mask.requires_render() {
            self.render_dirty = true;
        }
    }

    fn invalidate_host_for_scope(
        &mut self,
        view: Option<&ViewInstanceId>,
        mask: HostInvalidationMask,
    ) {
        self.capture_pending_ui_perf_scenario();
        record_ui_dirty_mask(mask);
        if let Some(view) = view {
            self.invalidation.invalidate_view(view, mask);
        } else {
            self.invalidation.invalidate(mask);
        }
        if view.is_none() && mask.requires_window_metrics() {
            self.window_metrics_dirty = true;
        }
        if view.is_none()
            && mask.intersects(
                HostInvalidationMask::LAYOUT.union(HostInvalidationMask::TREE_STRUCTURE),
            )
        {
            self.layout_dirty = true;
        }
        if view.is_none() && (mask.requires_presentation() || mask.requires_hit_test()) {
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

#[cfg(test)]
mod tests {
    #[test]
    fn window_metrics_keeps_its_own_legacy_dirty_domain() {
        let source = include_str!("dirty_flags.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("dirty flag production source");
        let layout_assignments = production
            .match_indices("self.layout_dirty = true")
            .map(|(index, _)| &production[index.saturating_sub(180)..index])
            .collect::<Vec<_>>();

        assert_eq!(layout_assignments.len(), 2);
        assert!(layout_assignments.iter().all(|context| {
            context.contains("HostInvalidationMask::LAYOUT")
                && context.contains("HostInvalidationMask::TREE_STRUCTURE")
                && !context.contains("requires_layout()")
        }));
    }
}
