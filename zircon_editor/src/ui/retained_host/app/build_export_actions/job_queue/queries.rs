use super::{DesktopExportJobPhase, DesktopExportJobQueue, DesktopExportJobSnapshot};

impl DesktopExportJobQueue {
    pub(in crate::ui::retained_host::app) fn is_profile_busy(&self, profile_name: &str) -> bool {
        self.active
            .as_ref()
            .is_some_and(|active| active.profile_name == profile_name)
            || self
                .pending
                .iter()
                .any(|pending| pending.profile_name == profile_name)
    }

    pub(in crate::ui::retained_host::app) fn snapshots(&self) -> Vec<DesktopExportJobSnapshot> {
        let mut snapshots = Vec::new();
        if let Some(active) = &self.active {
            snapshots.push(DesktopExportJobSnapshot {
                id: active.id,
                profile_name: active.profile_name.clone(),
                output_root: active.output_root.clone(),
                phase: if active.cancel.is_cancelled() {
                    DesktopExportJobPhase::CancelRequested
                } else {
                    DesktopExportJobPhase::Running
                },
                progress: active.progress.clone(),
            });
        }
        snapshots.extend(self.pending.iter().map(|pending| DesktopExportJobSnapshot {
            id: pending.id,
            profile_name: pending.profile_name.clone(),
            output_root: pending.output_root.clone(),
            phase: DesktopExportJobPhase::Queued,
            progress: None,
        }));
        snapshots
    }
}
