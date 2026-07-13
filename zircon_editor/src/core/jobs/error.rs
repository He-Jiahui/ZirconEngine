use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;

use thiserror::Error;

use super::JobId;

#[derive(Clone)]
pub struct JobFailure {
    source: Arc<dyn StdError + Send + Sync>,
}

impl JobFailure {
    pub fn new<E>(source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self {
            source: Arc::new(source),
        }
    }

    pub fn downcast_ref<E>(&self) -> Option<&E>
    where
        E: StdError + 'static,
    {
        self.source.downcast_ref::<E>()
    }
}

impl fmt::Debug for JobFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JobFailure")
            .field("source", &self.source.to_string())
            .finish()
    }
}

impl fmt::Display for JobFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.source, formatter)
    }
}

impl StdError for JobFailure {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.source.as_ref())
    }
}

impl PartialEq for JobFailure {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.source, &other.source)
    }
}

impl Eq for JobFailure {}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum JobError {
    #[error("editor job was cancelled")]
    Cancelled,
    #[error("editor job failed: {0}")]
    Failed(#[source] JobFailure),
    #[error("editor job panicked: {0}")]
    Panicked(String),
    #[error("editor job result channel closed before producing a result")]
    ResultChannelClosed,
}

impl JobError {
    pub fn failed<E>(source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self::Failed(JobFailure::new(source))
    }

    pub fn downcast_ref<E>(&self) -> Option<&E>
    where
        E: StdError + 'static,
    {
        match self {
            Self::Failed(failure) => failure.downcast_ref::<E>(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum JobSubmitError {
    #[error("editor job label must not be empty")]
    EmptyLabel,
    #[error("editor job system is shutting down and no longer accepts submissions")]
    ShuttingDown,
    #[error("editor job dependency {dependency:?} has expired from retained history")]
    ExpiredDependency { dependency: JobId },
    #[error("editor job dependency {dependency:?} is not registered")]
    UnknownDependency { dependency: JobId },
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum MutexGroupError {
    #[error("editor job mutex group must not be empty")]
    Empty,
    #[error("editor job mutex group segment is invalid: {value}")]
    Invalid { value: String },
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;

    use zircon_runtime::plugin::ExportBuildPlanError;

    use super::*;
    use crate::ui::host::EditorExportBuildError;

    #[test]
    fn failed_job_preserves_typed_export_error_for_downcast() {
        let job_error = JobError::failed(EditorExportBuildError::Plan(
            ExportBuildPlanError::MissingProfile {
                profile_name: "desktop".to_string(),
            },
        ));

        let export_error = job_error
            .downcast_ref::<EditorExportBuildError>()
            .expect("job failure must preserve the editor export error type");
        assert!(matches!(
            export_error,
            EditorExportBuildError::Plan(ExportBuildPlanError::MissingProfile { profile_name })
                if profile_name == "desktop"
        ));
        assert!(job_error.source().is_some());
    }

    #[test]
    fn cloned_failure_keeps_source_identity_without_text_equality() {
        let failure = JobError::failed(std::io::Error::other("typed source"));
        let clone = failure.clone();

        assert_eq!(failure, clone);
        assert!(failure.downcast_ref::<std::io::Error>().is_some());
    }
}
