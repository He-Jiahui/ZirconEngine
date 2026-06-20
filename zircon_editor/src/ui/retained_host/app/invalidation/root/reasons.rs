use super::super::HostInvalidationMask;
use super::HostInvalidationRoot;

impl HostInvalidationRoot {
    pub(in crate::ui::retained_host::app) fn take_recompute_reasons(
        &mut self,
    ) -> HostInvalidationMask {
        let reasons = self.pending_recompute;
        self.pending_recompute = HostInvalidationMask::NONE;
        reasons
    }

    pub(in crate::ui::retained_host::app) fn consume_recompute_reasons(
        &mut self,
        mask: HostInvalidationMask,
    ) -> HostInvalidationMask {
        let consumed = self.pending_recompute.intersection(mask);
        self.pending_recompute.remove(mask);
        consumed
    }

    pub(in crate::ui::retained_host::app) fn record_slow_path_rebuild(&mut self) -> u64 {
        self.slow_path_rebuilds += 1;
        self.slow_path_rebuilds
    }

    pub(in crate::ui::retained_host::app) fn record_render_rebuild(&mut self) -> u64 {
        self.render_rebuilds += 1;
        self.render_rebuilds
    }
}
