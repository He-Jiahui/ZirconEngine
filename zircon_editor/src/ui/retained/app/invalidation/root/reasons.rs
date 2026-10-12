use super::super::HostInvalidationMask;
use super::{HostInvalidationRoot, HostInvalidationTransaction};

impl HostInvalidationRoot {
    // 重算阶段取走已合并的展示事务；渲染提交仍以宿主 render_dirty 为入口，并可消费后到的渲染位。
    pub(in crate::ui::retained_host::app) fn take_recompute_transaction(
        &mut self,
    ) -> HostInvalidationTransaction {
        std::mem::take(&mut self.pending_recompute)
    }

    pub(in crate::ui::retained_host::app) fn consume_recompute_reasons(
        &mut self,
        mask: HostInvalidationMask,
    ) -> HostInvalidationMask {
        self.pending_recompute.consume(mask)
    }

    pub(in crate::ui::retained_host::app) fn has_pending_presentation_recompute(&self) -> bool {
        self.pending_recompute.requires_presentation_recompute()
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
