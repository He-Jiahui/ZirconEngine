use crate::ui::slint_host::HostInvalidationDiagnostics;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct HostInvalidationMask(u16);

impl HostInvalidationMask {
    pub(super) const NONE: Self = Self(0);
    pub(super) const LAYOUT: Self = Self(1 << 0);
    pub(super) const TREE_STRUCTURE: Self = Self(1 << 1);
    pub(super) const PRESENTATION_DATA: Self = Self(1 << 2);
    pub(super) const PAINT_ONLY: Self = Self(1 << 3);
    pub(super) const POINTER_HOVER: Self = Self(1 << 4);
    pub(super) const VIEWPORT_IMAGE: Self = Self(1 << 5);
    pub(super) const HIT_TEST: Self = Self(1 << 6);
    pub(super) const WINDOW_METRICS: Self = Self(1 << 7);
    pub(super) const RENDER: Self = Self(1 << 8);

    pub(super) const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub(super) const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub(super) fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub(super) fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    pub(super) const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(super) const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub(super) const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub(super) const fn requires_layout(self) -> bool {
        self.intersects(
            Self::LAYOUT
                .union(Self::TREE_STRUCTURE)
                .union(Self::WINDOW_METRICS),
        )
    }

    pub(super) const fn requires_presentation(self) -> bool {
        self.requires_layout() || self.intersects(Self::PRESENTATION_DATA)
    }

    pub(super) const fn requires_render(self) -> bool {
        self.intersects(Self::RENDER)
    }

    pub(super) const fn requires_window_metrics(self) -> bool {
        self.intersects(Self::WINDOW_METRICS)
    }

    pub(super) const fn requires_hit_test(self) -> bool {
        self.intersects(Self::HIT_TEST)
    }

    pub(super) const fn requires_host_recompute(self) -> bool {
        self.requires_layout()
            || self.requires_presentation()
            || self.requires_render()
            || self.requires_hit_test()
    }

    pub(super) fn from_dirty_flags(
        layout_dirty: bool,
        presentation_dirty: bool,
        window_metrics_dirty: bool,
        render_dirty: bool,
    ) -> Self {
        let mut mask = Self::NONE;
        if layout_dirty {
            mask.insert(Self::LAYOUT);
        }
        if presentation_dirty {
            mask.insert(Self::PRESENTATION_DATA);
        }
        if window_metrics_dirty {
            mask.insert(Self::WINDOW_METRICS);
        }
        if render_dirty {
            mask.insert(Self::RENDER);
        }
        mask
    }

    pub(super) fn summary(self) -> String {
        if self.is_empty() {
            return "none".to_string();
        }

        let mut names = Vec::new();
        if self.contains(Self::LAYOUT) {
            names.push("layout");
        }
        if self.contains(Self::TREE_STRUCTURE) {
            names.push("tree-structure");
        }
        if self.contains(Self::PRESENTATION_DATA) {
            names.push("presentation-data");
        }
        if self.contains(Self::PAINT_ONLY) {
            names.push("paint-only");
        }
        if self.contains(Self::POINTER_HOVER) {
            names.push("pointer-hover");
        }
        if self.contains(Self::VIEWPORT_IMAGE) {
            names.push("viewport-image");
        }
        if self.contains(Self::HIT_TEST) {
            names.push("hit-test");
        }
        if self.contains(Self::WINDOW_METRICS) {
            names.push("window-metrics");
        }
        if self.contains(Self::RENDER) {
            names.push("render");
        }
        names.join("|")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct HostInvalidationRoot {
    pending_recompute: HostInvalidationMask,
    total_requests: u64,
    layout_requests: u64,
    presentation_requests: u64,
    render_requests: u64,
    paint_only_requests: u64,
    hit_test_requests: u64,
    window_metrics_requests: u64,
    slow_path_rebuilds: u64,
    render_rebuilds: u64,
}

impl HostInvalidationRoot {
    pub(super) fn with_initial_full_rebuild() -> Self {
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

    pub(super) fn invalidate(&mut self, mask: HostInvalidationMask) {
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
            self.pending_recompute.insert(mask);
        }
    }

    pub(super) fn take_recompute_reasons(&mut self) -> HostInvalidationMask {
        let reasons = self.pending_recompute;
        self.pending_recompute = HostInvalidationMask::NONE;
        reasons
    }

    pub(super) fn consume_recompute_reasons(
        &mut self,
        mask: HostInvalidationMask,
    ) -> HostInvalidationMask {
        let consumed = self.pending_recompute.intersection(mask);
        self.pending_recompute.remove(mask);
        consumed
    }

    pub(super) fn record_slow_path_rebuild(&mut self) -> u64 {
        self.slow_path_rebuilds += 1;
        self.slow_path_rebuilds
    }

    pub(super) fn record_render_rebuild(&mut self) -> u64 {
        self.render_rebuilds += 1;
        self.render_rebuilds
    }

    pub(super) fn stats_summary(&self) -> String {
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

    pub(super) fn diagnostics_snapshot(&self) -> HostInvalidationDiagnostics {
        HostInvalidationDiagnostics {
            slow_path_rebuild_count: self.slow_path_rebuilds,
            render_rebuild_count: self.render_rebuilds,
            paint_only_request_count: self.paint_only_requests,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_invalidation_paint_only_does_not_require_slow_path() {
        let mask = HostInvalidationMask::PAINT_ONLY
            .union(HostInvalidationMask::POINTER_HOVER)
            .union(HostInvalidationMask::VIEWPORT_IMAGE);
        let mut root = HostInvalidationRoot::default();
        root.invalidate(mask);

        assert!(!mask.requires_host_recompute());
        assert!(root.take_recompute_reasons().is_empty());
        assert_eq!(root.paint_only_requests, 1);
    }

    #[test]
    fn host_invalidation_layout_implies_presentation_slow_path() {
        let mask = HostInvalidationMask::LAYOUT;

        assert!(mask.requires_layout());
        assert!(mask.requires_presentation());
        assert!(mask.requires_host_recompute());
    }

    #[test]
    fn host_invalidation_render_is_separate_from_presentation() {
        let mask = HostInvalidationMask::RENDER;

        assert!(mask.requires_render());
        assert!(!mask.requires_layout());
        assert!(!mask.requires_presentation());
        assert!(mask.requires_host_recompute());
    }

    #[test]
    fn host_invalidation_counts_and_drains_recompute_reasons() {
        let mut root = HostInvalidationRoot::default();
        root.invalidate(HostInvalidationMask::LAYOUT);
        root.invalidate(HostInvalidationMask::RENDER);

        let render = root.consume_recompute_reasons(HostInvalidationMask::RENDER);
        assert_eq!(render, HostInvalidationMask::RENDER);

        let remaining = root.take_recompute_reasons();
        assert!(remaining.contains(HostInvalidationMask::LAYOUT));
        assert!(!remaining.contains(HostInvalidationMask::RENDER));
        assert_eq!(root.total_requests, 2);
        assert_eq!(root.layout_requests, 1);
        assert_eq!(root.render_requests, 1);
    }

    #[test]
    fn host_invalidation_diagnostics_snapshot_exposes_paint_only_count() {
        let mut root = HostInvalidationRoot::default();
        root.invalidate(
            HostInvalidationMask::VIEWPORT_IMAGE.union(HostInvalidationMask::PAINT_ONLY),
        );
        root.record_slow_path_rebuild();
        root.record_render_rebuild();

        let diagnostics = root.diagnostics_snapshot();

        assert_eq!(diagnostics.slow_path_rebuild_count, 1);
        assert_eq!(diagnostics.render_rebuild_count, 1);
        assert_eq!(diagnostics.paint_only_request_count, 1);
    }
}
