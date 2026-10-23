use std::time::Instant;

use super::super::super::RetainedEditorHost;
use super::projection::{merge_probe_diagnostic, project_probe_projection};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn schedule_welcome_project_probe(&mut self) {
        self.startup_session.creation_validation = "Checking project location…".to_string();
        self.startup_session.can_open_existing = false;
        let now = Instant::now();
        self.welcome_project_probe
            .request(self.startup_session.draft.clone(), now);
        // Native field callbacks can run after tick; arm the first wake here.
        // Preserve an earlier close/watch/heartbeat deadline owned by the host.
        let deadline = [
            self.ui.lifecycle_frame_wake_deadline(),
            self.welcome_project_probe.poll_deadline(now),
        ]
        .into_iter()
        .flatten()
        .min();
        self.ui.set_lifecycle_frame_update(deadline);
    }

    pub(in crate::ui::retained_host::app) fn clear_welcome_project_probe(&mut self) {
        self.welcome_project_probe.clear();
    }

    pub(in crate::ui::retained_host::app) fn poll_welcome_project_probe(&mut self) {
        if let Err(error) = self
            .welcome_project_probe
            .submit_due(self.runtime.context().jobs(), Instant::now())
        {
            self.startup_session.creation_validation =
                format!("Project validation is unavailable: {error}");
            self.startup_session.can_open_existing = false;
            self.refresh_welcome_snapshot();
            return;
        }
        let Some((_generation, result)) = self.welcome_project_probe.poll() else {
            return;
        };
        let result = match result {
            Ok(result) => result,
            Err(error) => {
                self.startup_session.creation_validation =
                    format!("Project validation job failed: {error}");
                self.startup_session.can_open_existing = false;
                self.refresh_welcome_snapshot();
                return;
            }
        };
        let (can_open_existing, open_diagnostic) = project_probe_projection(result.open_probe);
        self.startup_session.creation_validation =
            merge_probe_diagnostic(result.creation_validation, open_diagnostic);
        self.startup_session.can_open_existing = can_open_existing;
        self.refresh_welcome_snapshot();
    }
}
