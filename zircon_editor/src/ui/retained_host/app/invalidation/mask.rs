mod requirements;
mod summary;

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
    pub(crate) const SHELL_CONTENT: Self = Self(1 << 9);
    pub(crate) const WORKBENCH_PROJECTION: Self = Self(1 << 10);

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
}
