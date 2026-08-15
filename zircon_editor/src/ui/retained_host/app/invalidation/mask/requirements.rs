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

    pub(crate) const fn requires_host_recompute(self) -> bool {
        self.requires_layout()
            || self.requires_presentation()
            || self.requires_hit_test()
            || self.intersects(Self::WORKBENCH_PROJECTION)
    }
}

#[cfg(test)]
mod tests {
    use super::HostInvalidationMask;

    #[test]
    fn workbench_projection_requires_a_host_commit_without_promoting_global_presentation() {
        let mask = HostInvalidationMask::WORKBENCH_PROJECTION;

        assert!(mask.requires_host_recompute());
        assert!(!mask.requires_layout());
        assert!(!mask.requires_presentation());
        assert!(!mask.requires_hit_test());
    }
}
