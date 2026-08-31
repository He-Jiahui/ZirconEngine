use std::fs;
use std::path::Path;

use serde::Serialize;
use zircon_runtime::core::resource::io::atomic_write;

const TERMINAL_OUTCOME_SCHEMA: &str = "zircon_shader_pbr_viewer_terminal_outcome_v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TerminalStatus {
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TerminalPhase {
    Startup,
    WindowCreation,
    PresenterCreation,
    SceneLoad,
    PipelineAdmission,
    PipelineReadiness,
    Render,
    ScreenshotWrite,
    GpuTimingWrite,
    RenderDocCapture,
    FramePresent,
    StatusPresent,
    EventLoop,
    TaskShutdown,
    UserCancel,
    Completed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TerminalErrorCategory {
    Startup,
    Platform,
    SceneLoad,
    Pipeline,
    Rendering,
    Artifact,
    Capture,
    Presentation,
    EventLoop,
    TaskShutdown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TerminalArtifactState {
    NotRequested,
    NotCommitted,
    Committed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TerminalCleanupState {
    RuntimeOwnersReleased,
    BackgroundLoaderCompletedAndJoined,
    BackgroundLoaderCancelledAndJoined,
    BackgroundLoaderShutdownTimedOut,
    BackgroundLoaderJoinPanicked,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TerminalOutcome {
    schema: &'static str,
    status: TerminalStatus,
    phase: TerminalPhase,
    error_category: Option<TerminalErrorCategory>,
    error_message: Option<String>,
    exit_code: u8,
    source_chain: Vec<String>,
    screenshot_artifact: TerminalArtifactState,
    gpu_timing_artifact: TerminalArtifactState,
    renderdoc_artifact: TerminalArtifactState,
    cleanup_state: TerminalCleanupState,
    cleanup_error: Option<String>,
}

impl TerminalOutcome {
    pub(crate) fn succeeded() -> Self {
        Self::new(
            TerminalStatus::Succeeded,
            TerminalPhase::Completed,
            None,
            None,
            0,
        )
    }

    pub(crate) fn cancelled() -> Self {
        Self::new(
            TerminalStatus::Cancelled,
            TerminalPhase::UserCancel,
            None,
            None,
            130,
        )
    }

    pub(crate) fn failure(
        phase: TerminalPhase,
        category: TerminalErrorCategory,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            TerminalStatus::Failed,
            phase,
            Some(category),
            Some(message.into()),
            category.exit_code(),
        )
    }

    fn new(
        status: TerminalStatus,
        phase: TerminalPhase,
        error_category: Option<TerminalErrorCategory>,
        error_message: Option<String>,
        exit_code: u8,
    ) -> Self {
        Self {
            schema: TERMINAL_OUTCOME_SCHEMA,
            status,
            phase,
            error_category,
            error_message,
            exit_code,
            source_chain: Vec::new(),
            screenshot_artifact: TerminalArtifactState::NotRequested,
            gpu_timing_artifact: TerminalArtifactState::NotRequested,
            renderdoc_artifact: TerminalArtifactState::NotRequested,
            cleanup_state: TerminalCleanupState::RuntimeOwnersReleased,
            cleanup_error: None,
        }
    }

    pub(crate) const fn status(&self) -> TerminalStatus {
        self.status
    }

    pub(crate) const fn exit_code(&self) -> u8 {
        self.exit_code
    }

    pub(crate) fn with_source_chain<T>(mut self, source_chain: impl IntoIterator<Item = T>) -> Self
    where
        T: Into<String>,
    {
        self.source_chain = source_chain.into_iter().map(Into::into).collect();
        self
    }

    pub(crate) const fn with_artifacts(
        mut self,
        screenshot_artifact: TerminalArtifactState,
        gpu_timing_artifact: TerminalArtifactState,
        renderdoc_artifact: TerminalArtifactState,
    ) -> Self {
        self.screenshot_artifact = screenshot_artifact;
        self.gpu_timing_artifact = gpu_timing_artifact;
        self.renderdoc_artifact = renderdoc_artifact;
        self
    }

    pub(crate) fn with_cleanup(
        mut self,
        cleanup_state: TerminalCleanupState,
        cleanup_error: Option<String>,
    ) -> Self {
        self.cleanup_state = cleanup_state;
        self.cleanup_error = cleanup_error;
        if let Some(cleanup_error) = self.cleanup_error.as_ref() {
            if self.status != TerminalStatus::Failed {
                self.status = TerminalStatus::Failed;
                self.phase = TerminalPhase::TaskShutdown;
                self.error_category = Some(TerminalErrorCategory::TaskShutdown);
                self.error_message = Some(cleanup_error.clone());
                self.exit_code = TerminalErrorCategory::TaskShutdown.exit_code();
            }
        }
        self
    }
}

impl TerminalErrorCategory {
    const fn exit_code(self) -> u8 {
        match self {
            Self::Startup => 11,
            Self::Platform => 12,
            Self::SceneLoad => 13,
            Self::Pipeline => 14,
            Self::Rendering => 15,
            Self::Artifact => 16,
            Self::Capture => 17,
            Self::Presentation => 18,
            Self::EventLoop => 19,
            Self::TaskShutdown => 20,
        }
    }
}

pub(crate) fn write_terminal_outcome(path: &Path, outcome: &TerminalOutcome) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| {
        format!(
            "terminal outcome path has no parent directory: {}",
            path.display()
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "create terminal outcome directory {}: {error}",
            parent.display()
        )
    })?;
    let mut bytes = serde_json::to_vec_pretty(outcome)
        .map_err(|error| format!("serialize terminal outcome: {error}"))?;
    bytes.push(b'\n');
    atomic_write(path, &bytes)
        .map_err(|error| format!("write terminal outcome {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use crate::work_paths::viewer_test_artifact_root;

    use super::{
        write_terminal_outcome, TerminalArtifactState, TerminalCleanupState, TerminalErrorCategory,
        TerminalOutcome, TerminalPhase, TerminalStatus,
    };

    #[test]
    fn terminal_failures_have_stable_nonzero_exit_codes() {
        for category in [
            TerminalErrorCategory::Startup,
            TerminalErrorCategory::Platform,
            TerminalErrorCategory::SceneLoad,
            TerminalErrorCategory::Pipeline,
            TerminalErrorCategory::Rendering,
            TerminalErrorCategory::Artifact,
            TerminalErrorCategory::Capture,
            TerminalErrorCategory::Presentation,
            TerminalErrorCategory::EventLoop,
            TerminalErrorCategory::TaskShutdown,
        ] {
            let outcome =
                TerminalOutcome::failure(TerminalPhase::Render, category, "simulated failure");

            assert_eq!(outcome.status(), TerminalStatus::Failed);
            assert_ne!(outcome.exit_code(), 0, "{category:?}");
        }
    }

    #[test]
    fn terminal_user_cancellation_is_distinct_from_success_and_failure() {
        let cancelled = TerminalOutcome::cancelled();

        assert_eq!(cancelled.status(), TerminalStatus::Cancelled);
        assert_ne!(
            cancelled.exit_code(),
            TerminalOutcome::succeeded().exit_code()
        );
        assert_ne!(
            cancelled.exit_code(),
            TerminalOutcome::failure(
                TerminalPhase::Render,
                TerminalErrorCategory::Rendering,
                "simulated failure",
            )
            .exit_code()
        );
    }

    #[test]
    fn terminal_record_is_atomic_and_retains_outcome_context() {
        let root = viewer_test_artifact_root("terminal-outcome");
        let path = root.join("viewer-terminal-outcome.json");
        let previous = TerminalOutcome::cancelled();
        write_terminal_outcome(&path, &previous).expect("write initial terminal record");

        let outcome = TerminalOutcome::failure(
            TerminalPhase::ScreenshotWrite,
            TerminalErrorCategory::Artifact,
            "cannot publish ready PNG",
        )
        .with_source_chain([
            "zircon_shader_pbr_viewer",
            "environment_only_pbr_preview",
            "hdri:E:/fixtures/lakes.hdr",
        ])
        .with_artifacts(
            TerminalArtifactState::NotCommitted,
            TerminalArtifactState::NotRequested,
            TerminalArtifactState::NotRequested,
        )
        .with_cleanup(
            TerminalCleanupState::BackgroundLoaderShutdownTimedOut,
            Some("scene loader exceeded the shutdown drain deadline".to_string()),
        );
        write_terminal_outcome(&path, &outcome).expect("replace terminal record atomically");

        let record: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).expect("read terminal record"))
                .expect("terminal record JSON");
        assert_eq!(
            record["schema"],
            "zircon_shader_pbr_viewer_terminal_outcome_v1"
        );
        assert_eq!(record["status"], "failed");
        assert_eq!(record["phase"], "screenshot_write");
        assert_eq!(record["error_category"], "artifact");
        assert_eq!(record["screenshot_artifact"], "not_committed");
        assert_eq!(
            record["cleanup_state"],
            "background_loader_shutdown_timed_out"
        );
        assert_eq!(
            record["cleanup_error"],
            "scene loader exceeded the shutdown drain deadline"
        );
        assert_eq!(record["source_chain"][2], "hdri:E:/fixtures/lakes.hdr");

        let staged = std::fs::read_dir(&root)
            .expect("read terminal record parent")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|candidate| candidate != &path)
            .collect::<Vec<_>>();
        assert!(
            staged.is_empty(),
            "terminal write left visible staging files: {staged:?}"
        );

        std::fs::remove_dir_all(root).expect("remove terminal outcome fixture");
    }

    #[test]
    fn shutdown_timeout_promotes_nonfatal_outcomes_to_a_task_shutdown_failure() {
        let outcome = TerminalOutcome::cancelled().with_cleanup(
            TerminalCleanupState::BackgroundLoaderShutdownTimedOut,
            Some("scene loader did not finish after cancellation".to_string()),
        );

        assert_eq!(outcome.status(), TerminalStatus::Failed);
        assert_eq!(outcome.phase, TerminalPhase::TaskShutdown);
        assert_eq!(
            outcome.error_category,
            Some(TerminalErrorCategory::TaskShutdown)
        );
        assert_eq!(outcome.exit_code(), 20);
        assert_eq!(
            outcome.cleanup_error.as_deref(),
            Some("scene loader did not finish after cancellation")
        );
    }

    #[test]
    fn cleanup_failure_does_not_replace_an_existing_render_failure() {
        let outcome = TerminalOutcome::failure(
            TerminalPhase::Render,
            TerminalErrorCategory::Rendering,
            "environment-only PBR render failed",
        )
        .with_cleanup(
            TerminalCleanupState::BackgroundLoaderShutdownTimedOut,
            Some("scene loader did not finish after cancellation".to_string()),
        );

        assert_eq!(outcome.status(), TerminalStatus::Failed);
        assert_eq!(outcome.phase, TerminalPhase::Render);
        assert_eq!(
            outcome.error_category,
            Some(TerminalErrorCategory::Rendering)
        );
        assert_eq!(outcome.exit_code(), 15);
        assert_eq!(
            outcome.cleanup_error.as_deref(),
            Some("scene loader did not finish after cancellation")
        );
    }
}
