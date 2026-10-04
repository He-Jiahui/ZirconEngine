//! 共享界面图像须按精确票据固定，GPU 回调或故障结算后才释放对应引用。
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use zr_rhi::SubmissionTicket;

use crate::ui_surface::WgpuUiImageInFlightPins;

#[derive(Clone, Default)]
pub(super) struct WgpuUiImageRetirementOwner {
    pending: Arc<Mutex<HashMap<SubmissionTicket, WgpuUiImageInFlightPins>>>,
}

impl WgpuUiImageRetirementOwner {
    pub(super) fn retain_batch(
        &self,
        retirements: impl IntoIterator<Item = (SubmissionTicket, WgpuUiImageInFlightPins)>,
    ) {
        let mut pending = self.lock_pending();
        for (ticket, pins) in retirements {
            let replaced = pending.insert(ticket, pins);
            debug_assert!(
                replaced.is_none(),
                "UI image pins attached twice to {ticket:?}"
            );
        }
    }

    pub(super) fn complete(&self, tickets: &[SubmissionTicket]) {
        let retired = {
            let mut pending = self.lock_pending();
            tickets
                .iter()
                .filter_map(|ticket| pending.remove(ticket))
                .collect::<Vec<_>>()
        };
        drop(retired);
    }

    /// Detach under the caller's queue serialization; the caller destroys the
    /// returned pins only after releasing its queue and state guards.
    pub(super) fn take_terminal_retirements(
        &self,
    ) -> HashMap<SubmissionTicket, WgpuUiImageInFlightPins> {
        let mut pending = self.lock_pending();
        std::mem::take(&mut *pending)
    }

    fn lock_pending(&self) -> MutexGuard<'_, HashMap<SubmissionTicket, WgpuUiImageInFlightPins>> {
        self.pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
