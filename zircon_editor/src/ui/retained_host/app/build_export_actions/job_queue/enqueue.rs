use std::path::PathBuf;

use crate::core::jobs::CancellationToken;

use super::{
    DesktopExportJobPhase, DesktopExportJobQueue, DesktopExportJobSnapshot, DesktopExportQueuedJob,
};
use zircon_runtime::asset::project::ProjectManifest;

impl DesktopExportJobQueue {
    pub(in crate::ui::retained_host::app) fn enqueue(
        &mut self,
        profile_name: impl Into<String>,
        project_root: PathBuf,
        manifest: ProjectManifest,
        output_root: PathBuf,
    ) -> DesktopExportJobSnapshot {
        let id = self.next_id;
        self.next_id += 1;
        let profile_name = profile_name.into();
        let snapshot = DesktopExportJobSnapshot {
            id,
            profile_name: profile_name.clone(),
            output_root: output_root.clone(),
            phase: DesktopExportJobPhase::Queued,
            progress: None,
        };
        self.pending.push_back(DesktopExportQueuedJob {
            id,
            profile_name,
            project_root,
            manifest,
            output_root,
            cancel: CancellationToken::default(),
        });
        snapshot
    }
}
