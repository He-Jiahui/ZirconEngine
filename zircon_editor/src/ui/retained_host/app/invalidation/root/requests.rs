use super::super::HostInvalidationMask;
use super::{HostInvalidationRoot, HostInvalidationScope};
use crate::ui::retained_host::HostShellContentScope;
use crate::ui::workbench::view::ViewInstanceId;

impl HostInvalidationRoot {
    pub(in crate::ui::retained_host::app) fn with_initial_full_rebuild() -> Self {
        let mut root = Self::default();
        root.invalidate(
            HostInvalidationMask::LAYOUT
                .union(HostInvalidationMask::WINDOW_METRICS)
                .union(HostInvalidationMask::PRESENTATION_DATA)
                .union(HostInvalidationMask::HIT_TEST)
                .union(HostInvalidationMask::RENDER),
        );
        root
    }

    pub(in crate::ui::retained_host::app) fn invalidate(&mut self, mask: HostInvalidationMask) {
        self.invalidate_scoped(HostInvalidationScope::All, mask);
    }

    pub(in crate::ui::retained_host::app) fn invalidate_view(
        &mut self,
        view: ViewInstanceId,
        mask: HostInvalidationMask,
    ) {
        self.invalidate_scoped(HostInvalidationScope::View(view), mask);
    }

    pub(in crate::ui::retained_host::app) fn invalidate_shell_content(
        &mut self,
        scope: HostShellContentScope,
        mask: HostInvalidationMask,
    ) {
        self.invalidate_scoped(HostInvalidationScope::ShellContent(scope), mask);
    }

    fn invalidate_scoped(&mut self, scope: HostInvalidationScope, mask: HostInvalidationMask) {
        if mask.is_empty() {
            return;
        }

        self.total_requests += 1;
        if mask.requires_layout() {
            self.layout_requests += 1;
        }
        if mask.requires_presentation() {
            self.presentation_requests += 1;
        }
        if mask.requires_render() {
            self.render_requests += 1;
        }
        if mask.intersects(
            HostInvalidationMask::PAINT_ONLY
                .union(HostInvalidationMask::POINTER_HOVER)
                .union(HostInvalidationMask::VIEWPORT_IMAGE),
        ) {
            self.paint_only_requests += 1;
        }
        if mask.requires_hit_test() {
            self.hit_test_requests += 1;
        }
        if mask.requires_window_metrics() {
            self.window_metrics_requests += 1;
        }
        if mask.requires_host_recompute() {
            self.pending_recompute.insert(scope, mask);
        }
    }
}
