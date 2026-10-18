//! 把截止时仍未完成的任务身份、分类及标签交给关闭边界，供宿主报告或继续处理；该诊断结果本身不是已停止工作的确认。
use super::{JobCategory, JobId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnfinishedEditorJob {
    id: JobId,
    label: String,
    category: JobCategory,
}

impl UnfinishedEditorJob {
    pub(super) fn new(id: JobId, label: String, category: JobCategory) -> Self {
        Self {
            id,
            label,
            category,
        }
    }

    pub fn id(&self) -> JobId {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn category(&self) -> JobCategory {
        self.category
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorJobShutdownState {
    NotRequested,
    Pending,
    Quiescent,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorJobShutdownFailure {
    CleanupSubmissionPanicked { message: String },
    JobSubmissionPanicked { job: JobId, message: String },
    JobTaskFailed { job: JobId, message: String },
    CleanupTaskFailed,
    CleanupCancelled,
}

impl EditorJobShutdownFailure {
    pub fn message(&self) -> &str {
        match self {
            Self::CleanupSubmissionPanicked { message }
            | Self::JobSubmissionPanicked { message, .. }
            | Self::JobTaskFailed { message, .. } => message,
            Self::CleanupTaskFailed => "editor shutdown cleanup task failed",
            Self::CleanupCancelled => "editor shutdown cleanup task was cancelled",
        }
    }
}

/// An unfinished-job list alone does not establish shutdown quiescence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorJobShutdownProgress {
    state: EditorJobShutdownState,
    unfinished_jobs: Vec<UnfinishedEditorJob>,
    completion_callbacks_pending: bool,
    owned_tasks_pending: bool,
    cleanup_pending: bool,
    cleanup_failure: Option<EditorJobShutdownFailure>,
}

impl EditorJobShutdownProgress {
    pub(super) fn new(
        requested: bool,
        unfinished_jobs: Vec<UnfinishedEditorJob>,
        completion_callbacks_pending: bool,
        owned_tasks_pending: bool,
        cleanup_pending: bool,
        cleanup_failure: Option<EditorJobShutdownFailure>,
    ) -> Self {
        let state = if !requested {
            EditorJobShutdownState::NotRequested
        } else if cleanup_failure.is_some() {
            EditorJobShutdownState::Failed
        } else if unfinished_jobs.is_empty()
            && !completion_callbacks_pending
            && !owned_tasks_pending
            && !cleanup_pending
        {
            EditorJobShutdownState::Quiescent
        } else {
            EditorJobShutdownState::Pending
        };
        Self {
            state,
            unfinished_jobs,
            completion_callbacks_pending,
            owned_tasks_pending,
            cleanup_pending,
            cleanup_failure,
        }
    }

    pub fn state(&self) -> EditorJobShutdownState {
        self.state
    }

    pub fn unfinished_jobs(&self) -> &[UnfinishedEditorJob] {
        &self.unfinished_jobs
    }

    pub fn completion_callbacks_pending(&self) -> bool {
        self.completion_callbacks_pending
    }

    pub fn owned_tasks_pending(&self) -> bool {
        self.owned_tasks_pending
    }

    pub fn cleanup_pending(&self) -> bool {
        self.cleanup_pending
    }

    pub fn cleanup_failure_kind(&self) -> Option<&EditorJobShutdownFailure> {
        self.cleanup_failure.as_ref()
    }

    pub fn cleanup_failure(&self) -> Option<&str> {
        self.cleanup_failure
            .as_ref()
            .map(EditorJobShutdownFailure::message)
    }
}
