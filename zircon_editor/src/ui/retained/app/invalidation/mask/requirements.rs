use super::HostInvalidationMask;

impl HostInvalidationMask {
    pub(crate) const fn requires_layout(self) -> bool {
        self.intersects(
            Self::LAYOUT
                .union(Self::TREE_STRUCTURE)
                .union(Self::WINDOW_METRICS),
        )
    }

    pub(crate) const fn requires_presentation(self) -> bool {
        self.requires_layout()
            || self.intersects(Self::PRESENTATION_DATA.union(Self::SHELL_CONTENT))
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

    // 仅布局、展示、命中或工作台投影要求宿主重算；渲染和纯重绘分别由帧后段处理。
    pub(crate) const fn requires_host_recompute(self) -> bool {
        self.requires_layout()
            || self.requires_presentation()
            || self.requires_hit_test()
            || self.intersects(Self::WORKBENCH_PROJECTION)
    }
}

#[cfg(test)]
#[path = "tests/requirements.rs"]
mod tests;
