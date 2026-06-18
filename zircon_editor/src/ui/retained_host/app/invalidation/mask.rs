#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HostInvalidationMask(u16);

impl HostInvalidationMask {
    pub(crate) const NONE: Self = Self(0);
    pub(crate) const LAYOUT: Self = Self(1 << 0);
    pub(crate) const TREE_STRUCTURE: Self = Self(1 << 1);
    pub(crate) const PRESENTATION_DATA: Self = Self(1 << 2);
    pub(crate) const PAINT_ONLY: Self = Self(1 << 3);
    pub(crate) const POINTER_HOVER: Self = Self(1 << 4);
    pub(crate) const VIEWPORT_IMAGE: Self = Self(1 << 5);
    pub(crate) const HIT_TEST: Self = Self(1 << 6);
    pub(crate) const WINDOW_METRICS: Self = Self(1 << 7);
    pub(crate) const RENDER: Self = Self(1 << 8);

    pub(crate) const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub(crate) const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub(crate) fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub(crate) fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    pub(crate) const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub(crate) const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub(crate) const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub(crate) const fn requires_layout(self) -> bool {
        self.intersects(
            Self::LAYOUT
                .union(Self::TREE_STRUCTURE)
                .union(Self::WINDOW_METRICS),
        )
    }

    pub(crate) const fn requires_presentation(self) -> bool {
        self.requires_layout() || self.intersects(Self::PRESENTATION_DATA)
    }

    pub(crate) const fn requires_render(self) -> bool {
        self.intersects(Self::RENDER)
    }

    pub(crate) const fn requires_window_metrics(self) -> bool {
        self.intersects(Self::WINDOW_METRICS)
    }

    pub(crate) const fn requires_hit_test(self) -> bool {
        self.intersects(Self::HIT_TEST)
    }

    pub(crate) const fn requires_host_recompute(self) -> bool {
        self.requires_layout()
            || self.requires_presentation()
            || self.requires_render()
            || self.requires_hit_test()
    }

    pub(crate) fn from_dirty_flags(
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

    pub(crate) fn summary(self) -> String {
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
