use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;
use zircon_runtime::core::resource::io::atomic_write_new;

use super::{AutosaveDocumentId, RestoreAction, RestoreCandidate, RestorePlan, RestoreStartup};

const RECOVERY_OUTPUT_DIRECTORY: &str = "recovered";
const RECOVERED_COPY_DIRECTORY: &str = "restore";
const COMPARISON_COPY_DIRECTORY: &str = "comparison";
const COPY_NAME_ALLOCATION_ATTEMPTS: u32 = 64;

/// Executes a validated recovery plan without ever replacing an authoritative source file.
pub struct RestoreExecutor {
    project_root: PathBuf,
}

impl RestoreExecutor {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }

    pub fn execute(
        &self,
        startup: &RestoreStartup,
        plan: &RestorePlan,
    ) -> Result<RestoreExecutionReport, RestoreExecutionError> {
        let mut candidates = BTreeMap::new();
        for candidate in startup.candidates() {
            let document = candidate.document().clone();
            if candidates.insert(document.clone(), candidate).is_some() {
                return Err(RestoreExecutionError::DuplicateCandidate {
                    document: document.as_str().to_string(),
                });
            }
        }
        for resolution in plan.resolutions() {
            if !candidates.contains_key(resolution.document()) {
                return Err(RestoreExecutionError::UnexpectedResolution {
                    document: resolution.document().as_str().to_string(),
                });
            }
        }

        let mut records = Vec::with_capacity(plan.resolutions().len());
        for resolution in plan.resolutions() {
            let Some(candidate) = candidates.get(resolution.document()) else {
                return Err(RestoreExecutionError::UnexpectedResolution {
                    document: resolution.document().as_str().to_string(),
                });
            };
            records.push(RestoreExecutionRecord {
                resolution: resolution.clone(),
                result: self.execute_resolution(candidate, resolution.action()),
            });
        }

        Ok(RestoreExecutionReport { records })
    }

    fn execute_resolution(
        &self,
        candidate: &RestoreCandidate,
        action: RestoreAction,
    ) -> Result<RestoreExecutionOutcome, RestoreDocumentExecutionError> {
        self.validate_candidate_path(candidate)?;
        match action {
            RestoreAction::RestoreAutosave => self
                .materialize_copy(candidate, RECOVERED_COPY_DIRECTORY)
                .map(RestoreExecutionOutcome::RecoveredCopy),
            RestoreAction::OpenComparison => self
                .materialize_copy(candidate, COMPARISON_COPY_DIRECTORY)
                .map(RestoreExecutionOutcome::ComparisonCopy),
            RestoreAction::DiscardAutosave => {
                self.discard_candidate(candidate)?;
                Ok(RestoreExecutionOutcome::Discarded {
                    document: candidate.document().clone(),
                })
            }
        }
    }

    fn materialize_copy(
        &self,
        candidate: &RestoreCandidate,
        purpose: &str,
    ) -> Result<RecoveredDocumentCopy, RestoreDocumentExecutionError> {
        let bytes = fs::read(candidate.autosave_path()).map_err(|source| {
            RestoreDocumentExecutionError::Io {
                operation: "read autosave snapshot for recovery",
                path: candidate.autosave_path().to_path_buf(),
                source,
            }
        })?;
        let directory = self
            .project_root
            .join(".zircon")
            .join(RECOVERY_OUTPUT_DIRECTORY)
            .join(purpose);
        for attempt in 0..COPY_NAME_ALLOCATION_ATTEMPTS {
            let path = recovered_copy_path(&directory, candidate, attempt);
            match atomic_write_new(&path, &bytes) {
                Ok(()) => {
                    return Ok(RecoveredDocumentCopy {
                        document: candidate.document().clone(),
                        source_path: candidate.source_path().to_path_buf(),
                        recovered_path: path,
                    });
                }
                Err(source) if source.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(source) => {
                    return Err(RestoreDocumentExecutionError::Io {
                        operation: "publish recovered document copy",
                        path,
                        source,
                    });
                }
            }
        }
        Err(RestoreDocumentExecutionError::CopyNameExhausted {
            document: candidate.document().as_str().to_string(),
            directory,
        })
    }

    fn discard_candidate(
        &self,
        candidate: &RestoreCandidate,
    ) -> Result<(), RestoreDocumentExecutionError> {
        let directory = self.autosave_document_directory(candidate.document());
        match fs::remove_dir_all(&directory) {
            Ok(()) => Ok(()),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(RestoreDocumentExecutionError::Io {
                operation: "discard autosave recovery document",
                path: directory,
                source,
            }),
        }
    }

    fn autosave_document_directory(&self, document: &AutosaveDocumentId) -> PathBuf {
        self.project_root
            .join(".zircon")
            .join("autosave")
            .join(document.as_str())
    }

    fn validate_candidate_path(
        &self,
        candidate: &RestoreCandidate,
    ) -> Result<(), RestoreDocumentExecutionError> {
        let directory = self.autosave_document_directory(candidate.document());
        if candidate.autosave_path().starts_with(&directory) {
            Ok(())
        } else {
            Err(RestoreDocumentExecutionError::InvalidCandidatePath {
                document: candidate.document().as_str().to_string(),
                path: candidate.autosave_path().to_path_buf(),
            })
        }
    }
}

fn recovered_copy_path(directory: &Path, candidate: &RestoreCandidate, attempt: u32) -> PathBuf {
    let extension = candidate
        .source_path()
        .extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| !extension.is_empty())
        .or_else(|| {
            candidate
                .autosave_path()
                .extension()
                .and_then(|extension| extension.to_str())
                .filter(|extension| !extension.is_empty())
        });
    let suffix = if attempt == 0 {
        String::new()
    } else {
        format!("-{attempt}")
    };
    let file_name = match extension {
        Some(extension) => format!(
            "{}-recovered{suffix}.{extension}",
            candidate.document().as_str()
        ),
        None => format!("{}-recovered{suffix}", candidate.document().as_str()),
    };
    directory.join(file_name)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveredDocumentCopy {
    document: AutosaveDocumentId,
    source_path: PathBuf,
    recovered_path: PathBuf,
}

impl RecoveredDocumentCopy {
    pub fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn recovered_path(&self) -> &Path {
        &self.recovered_path
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestoreExecutionOutcome {
    RecoveredCopy(RecoveredDocumentCopy),
    ComparisonCopy(RecoveredDocumentCopy),
    Discarded { document: AutosaveDocumentId },
}

impl RestoreExecutionOutcome {
    pub fn document(&self) -> &AutosaveDocumentId {
        match self {
            Self::RecoveredCopy(copy) | Self::ComparisonCopy(copy) => copy.document(),
            Self::Discarded { document } => document,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreExecutionRetryability {
    Retryable,
    RequiresOperatorIntervention,
}

#[derive(Debug, Error)]
pub enum RestoreDocumentExecutionError {
    #[error("recovery candidate for `{document}` escaped its autosave directory: {path}")]
    InvalidCandidatePath { document: String, path: PathBuf },
    #[error("could not allocate a recovered copy name for `{document}` below {directory}")]
    CopyNameExhausted {
        document: String,
        directory: PathBuf,
    },
    #[error("failed to {operation} `{path}`: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

impl RestoreDocumentExecutionError {
    pub const fn retryability(&self) -> RestoreExecutionRetryability {
        match self {
            Self::InvalidCandidatePath { .. } => {
                RestoreExecutionRetryability::RequiresOperatorIntervention
            }
            Self::CopyNameExhausted { .. } | Self::Io { .. } => {
                RestoreExecutionRetryability::Retryable
            }
        }
    }
}

#[derive(Debug)]
pub struct RestoreExecutionRecord {
    resolution: RestoreResolution,
    result: Result<RestoreExecutionOutcome, RestoreDocumentExecutionError>,
}

impl RestoreExecutionRecord {
    pub fn resolution(&self) -> &RestoreResolution {
        &self.resolution
    }

    pub fn document(&self) -> &AutosaveDocumentId {
        self.resolution.document()
    }

    pub const fn action(&self) -> RestoreAction {
        self.resolution.action()
    }

    pub fn outcome(&self) -> Option<&RestoreExecutionOutcome> {
        self.result.as_ref().ok()
    }

    pub fn failure(&self) -> Option<&RestoreDocumentExecutionError> {
        self.result.as_ref().err()
    }

    pub fn retryability(&self) -> Option<RestoreExecutionRetryability> {
        self.failure()
            .map(RestoreDocumentExecutionError::retryability)
    }
}

#[derive(Debug, Default)]
pub struct RestoreExecutionReport {
    records: Vec<RestoreExecutionRecord>,
}

impl RestoreExecutionReport {
    pub fn records(&self) -> &[RestoreExecutionRecord] {
        &self.records
    }

    pub fn success_count(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.outcome().is_some())
            .count()
    }

    pub fn failure_count(&self) -> usize {
        self.records.len().saturating_sub(self.success_count())
    }

    pub fn has_failures(&self) -> bool {
        self.records.iter().any(|record| record.failure().is_some())
    }

    pub fn retryable_resolutions(&self) -> Vec<RestoreResolution> {
        self.records
            .iter()
            .filter(|record| record.retryability() == Some(RestoreExecutionRetryability::Retryable))
            .map(|record| record.resolution().clone())
            .collect()
    }
}

#[derive(Debug, Error)]
pub enum RestoreExecutionError {
    #[error("recovery startup contains duplicate candidate `{document}`")]
    DuplicateCandidate { document: String },
    #[error("recovery plan referenced unexpected document `{document}`")]
    UnexpectedResolution { document: String },
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::SystemTime;

    use zircon_runtime_interface::project::session_lock::ProjectSessionPrincipalV1;
    use zircon_runtime_interface::project::{
        ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
    };
    use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

    use super::{
        RestoreDocumentExecutionError, RestoreExecutionOutcome, RestoreExecutionRetryability,
        RestoreExecutor,
    };
    use crate::core::recovery::{
        AutosaveDocumentId, RestoreAction, RestoreCandidate, RestoreFlow, RestoreFreshness,
        RestoreResolution, SessionAdmissionRequest, SessionGuard, SessionGuardAdmission,
    };

    #[test]
    fn restore_materializes_a_copy_without_changing_the_authoritative_source() {
        let root = temporary_root("restore-copy");
        let source_path = root.join("assets").join("scene.zscene");
        let autosave_path = root
            .join(".zircon")
            .join("autosave")
            .join("scene_main")
            .join("1.zscene");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::create_dir_all(autosave_path.parent().unwrap()).unwrap();
        fs::write(&source_path, "authoritative source").unwrap();
        fs::write(&autosave_path, "recovered autosave").unwrap();

        let lock = residual_lock(&root);
        let document = AutosaveDocumentId::parse("scene_main").unwrap();
        let candidate = RestoreCandidate::new(
            document.clone(),
            source_path.clone(),
            autosave_path,
            RestoreFreshness::SnapshotAheadOfSource,
        );
        let startup = RestoreFlow::detect(lock, [candidate]).unwrap();
        let plan = RestoreFlow::plan(
            &startup,
            [RestoreResolution::new(
                document,
                RestoreAction::RestoreAutosave,
            )],
        )
        .unwrap();

        let report = RestoreExecutor::new(&root)
            .execute(&startup, &plan)
            .unwrap();

        assert_eq!(
            fs::read_to_string(&source_path).unwrap(),
            "authoritative source"
        );
        let RestoreExecutionOutcome::RecoveredCopy(copy) = report.records()[0]
            .outcome()
            .expect("restore should succeed")
        else {
            panic!("expected recovered copy outcome");
        };
        assert_eq!(
            fs::read_to_string(copy.recovered_path()).unwrap(),
            "recovered autosave"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn discard_removes_only_the_selected_document_recovery_directory() {
        let root = temporary_root("restore-discard");
        let document = AutosaveDocumentId::parse("scene_main").unwrap();
        let autosave_path = root
            .join(".zircon")
            .join("autosave")
            .join(document.as_str())
            .join("1.zscene");
        fs::create_dir_all(autosave_path.parent().unwrap()).unwrap();
        fs::write(&autosave_path, "discarded autosave").unwrap();

        let lock = residual_lock(&root);
        let candidate = RestoreCandidate::new(
            document.clone(),
            root.join("assets").join("scene.zscene"),
            autosave_path.clone(),
            RestoreFreshness::SourceMissing,
        );
        let startup = RestoreFlow::detect(lock, [candidate]).unwrap();
        let plan = RestoreFlow::plan(
            &startup,
            [RestoreResolution::new(
                document,
                RestoreAction::DiscardAutosave,
            )],
        )
        .unwrap();

        let report = RestoreExecutor::new(&root)
            .execute(&startup, &plan)
            .unwrap();

        assert!(matches!(
            report.records()[0].outcome(),
            Some(RestoreExecutionOutcome::Discarded { .. })
        ));
        assert!(!autosave_path.parent().unwrap().exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn execution_reports_every_document_after_retryable_and_permanent_failures() {
        let root = temporary_root("restore-partial-report");
        let retry_document = AutosaveDocumentId::parse("a_retry").unwrap();
        let rejected_document = AutosaveDocumentId::parse("b_rejected").unwrap();
        let restored_document = AutosaveDocumentId::parse("c_restored").unwrap();
        let retry_path = root
            .join(".zircon")
            .join("autosave")
            .join(retry_document.as_str())
            .join("missing.zscene");
        let rejected_path = root.join("outside-autosave").join("snapshot.zscene");
        let restored_path = root
            .join(".zircon")
            .join("autosave")
            .join(restored_document.as_str())
            .join("1.zscene");
        fs::create_dir_all(restored_path.parent().unwrap()).unwrap();
        fs::write(&restored_path, "recover me").unwrap();

        let startup = RestoreFlow::detect(
            residual_lock(&root),
            [
                RestoreCandidate::new(
                    retry_document.clone(),
                    root.join("assets/a_retry.zscene"),
                    retry_path,
                    RestoreFreshness::SnapshotAheadOfSource,
                ),
                RestoreCandidate::new(
                    rejected_document.clone(),
                    root.join("assets/b_rejected.zscene"),
                    rejected_path,
                    RestoreFreshness::SnapshotAheadOfSource,
                ),
                RestoreCandidate::new(
                    restored_document.clone(),
                    root.join("assets/c_restored.zscene"),
                    restored_path,
                    RestoreFreshness::SnapshotAheadOfSource,
                ),
            ],
        )
        .unwrap();
        let plan = RestoreFlow::plan(
            &startup,
            [
                RestoreResolution::new(retry_document.clone(), RestoreAction::RestoreAutosave),
                RestoreResolution::new(rejected_document.clone(), RestoreAction::RestoreAutosave),
                RestoreResolution::new(restored_document.clone(), RestoreAction::RestoreAutosave),
            ],
        )
        .unwrap();

        let report = RestoreExecutor::new(&root)
            .execute(&startup, &plan)
            .unwrap();

        assert_eq!(report.records().len(), 3);
        assert_eq!(report.success_count(), 1);
        assert_eq!(report.failure_count(), 2);
        assert!(report.has_failures());
        assert_eq!(report.records()[0].document(), &retry_document);
        assert!(matches!(
            report.records()[0].failure(),
            Some(RestoreDocumentExecutionError::Io { .. })
        ));
        assert_eq!(
            report.records()[0].retryability(),
            Some(RestoreExecutionRetryability::Retryable)
        );
        assert_eq!(report.records()[1].document(), &rejected_document);
        assert!(matches!(
            report.records()[1].failure(),
            Some(RestoreDocumentExecutionError::InvalidCandidatePath { .. })
        ));
        assert_eq!(
            report.records()[1].retryability(),
            Some(RestoreExecutionRetryability::RequiresOperatorIntervention)
        );
        assert_eq!(report.records()[2].document(), &restored_document);
        assert!(report.records()[2].outcome().is_some());

        let retryable = report.retryable_resolutions();
        assert_eq!(retryable.len(), 1);
        assert_eq!(retryable[0].document(), &retry_document);
        assert_eq!(retryable[0].action(), RestoreAction::RestoreAutosave);
        let retry_plan = RestoreFlow::retry_plan(&plan, retryable)
            .unwrap()
            .expect("one retryable failure should produce a retry plan");
        assert_eq!(retry_plan.resolutions().len(), 1);
        assert_eq!(retry_plan.resolutions()[0].document(), &retry_document);
        assert!(matches!(
            RestoreFlow::retry_plan(
                &plan,
                [RestoreResolution::new(
                    retry_document,
                    RestoreAction::DiscardAutosave,
                )],
            ),
            Err(crate::core::recovery::RestoreFlowError::ChangedRetryAction { .. })
        ));

        fs::remove_dir_all(root).unwrap();
    }

    fn temporary_root(label: &str) -> std::path::PathBuf {
        std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!(
                "zircon-editor-{label}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ))
    }

    fn residual_lock(
        project_root: &std::path::Path,
    ) -> crate::core::recovery::SessionLockInspection {
        let operation = ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
            .allocate()
            .expect("fixture operation id");
        let admission = SessionAdmissionRequest::new(
            operation,
            ProjectSessionPrincipalV1::Welcome,
            ZrRuntimeBuildSetId::parse(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .expect("fixture BuildSet"),
        );
        let guard =
            match SessionGuard::claim(project_root, &admission).expect("fixture session claim") {
                SessionGuardAdmission::Acquired(guard) => guard,
                SessionGuardAdmission::Active { .. } | SessionGuardAdmission::Residual(_) => {
                    panic!("fresh fixture root must acquire a session guard")
                }
            };
        let inspection =
            SessionGuard::inspect(project_root).expect("inspect residual fixture lock");
        drop(guard);
        inspection
    }
}
