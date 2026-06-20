use std::sync::atomic::Ordering;

use super::super::DesktopExportExecutionSummary;
use super::{DesktopExportJobPhase, DesktopExportJobQueue, DesktopExportJobSnapshot};

#[derive(Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) enum DesktopExportCancellation {
    NotFound,
    PendingCancelled(DesktopExportExecutionSummary),
    ActiveCancelRequested(DesktopExportJobSnapshot),
}

impl DesktopExportJobQueue {
    pub(in crate::ui::retained_host::app) fn cancel_profile(
        &mut self,
        profile_name: &str,
    ) -> DesktopExportCancellation {
        if let Some(index) = self
            .pending
            .iter()
            .position(|pending| pending.profile_name == profile_name)
        {
            let pending = self
                .pending
                .remove(index)
                .expect("pending job index came from this queue");
            pending.cancel_requested.store(true, Ordering::SeqCst);
            return DesktopExportCancellation::PendingCancelled(
                DesktopExportExecutionSummary::cancelled(
                    pending.profile_name,
                    pending.output_root,
                    "Queued export was cancelled before it started".to_string(),
                ),
            );
        }

        if let Some(active) = self
            .active
            .as_ref()
            .filter(|active| active.profile_name == profile_name)
        {
            active.cancel_requested.store(true, Ordering::SeqCst);
            return DesktopExportCancellation::ActiveCancelRequested(DesktopExportJobSnapshot {
                id: active.id,
                profile_name: active.profile_name.clone(),
                output_root: active.output_root.clone(),
                phase: DesktopExportJobPhase::CancelRequested,
                progress: active.progress.clone(),
            });
        }

        DesktopExportCancellation::NotFound
    }
}
