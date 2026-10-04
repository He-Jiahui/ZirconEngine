//! 查看器给运行工具的稳定终态记录与退出码契约。
//! 主错误、证据发布状态和退出清理分别记录，避免清理错误掩盖最初失败。

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

/// 工件请求与发布的终态快照；失败或取消不应被误读为已提交证据。
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

/// 运行工具消费的主结果、输入来源、工件状态与清理结果；清理失败保留已有主失败。
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

    /// 附加退出阶段结果；原成功或取消会因清理错误转失败，已有主错误保持不变。
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

/// 主入口在终态确定后调用；原子替换记录，发布错误交由最外层进程错误边界处理。
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
#[path = "tests/terminal_outcome.rs"]
mod tests;
