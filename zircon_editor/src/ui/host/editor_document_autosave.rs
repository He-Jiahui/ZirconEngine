use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::ProjectPaths;

use crate::core::editor_message::DocumentId;
use crate::core::extension::{DocumentSaveReport, SaveReason};
use crate::core::jobs::{
    EditorJob, EditorJobSpec, JobCategory, JobContext, JobError, JobPriority, MutexGroup,
};
use crate::core::recovery::{
    AutosaveDocumentId, AutosaveDocumentRequest, AutosaveExtension, AutosaveJobPolicy,
    AutosaveSnapshot, AutosaveSnapshotProvenance, AutosaveSnapshotSource, AutosaveSourceDigest,
    AutosaveSourcePath,
};

use super::{EditorError, EditorManager};

#[derive(Clone, Debug)]
pub(crate) struct EditorAutosaveDocumentIdentity {
    document: AutosaveDocumentId,
    source_path: AutosaveSourcePath,
    physical_source_path: PathBuf,
}

impl EditorAutosaveDocumentIdentity {
    pub(crate) fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }
}

impl EditorManager {
    pub(crate) fn autosave_document_identity(
        &self,
        document: DocumentId,
        project_root: &Path,
    ) -> Result<EditorAutosaveDocumentIdentity, EditorError> {
        let source_path = self.host.document_autosave_source_path(document)?;
        editor_autosave_document_identity(project_root, source_path)
    }

    pub(crate) fn autosave_document_request(
        self: &Arc<Self>,
        document: DocumentId,
        dirty_generation: u64,
        identity: EditorAutosaveDocumentIdentity,
    ) -> Result<AutosaveDocumentRequest, EditorError> {
        let save_mutex = document_save_mutex_group(&identity.physical_source_path)?;
        Ok(AutosaveDocumentRequest::new(
            identity.document.clone(),
            AutosaveJobPolicy::for_save_mutex(save_mutex),
            Arc::new(EditorDocumentAutosaveSource {
                manager: Arc::downgrade(self),
                document,
                dirty_generation,
                sequence: next_autosave_sequence(),
                source_path: identity.source_path,
                physical_source_path: identity.physical_source_path,
            }),
        ))
    }
}

pub(super) fn document_save_mutex_group(source_path: &Path) -> Result<MutexGroup, EditorError> {
    let source = ProjectPaths::resolve_path(source_path)
        .map(|path| path.into_operation_path())
        .map_err(|error| EditorError::Project(error.to_string()))?;
    let source = source.to_string_lossy();
    MutexGroup::parse(format!(
        "save_document_{}",
        blake3::hash(source.as_bytes()).to_hex()
    ))
    .map_err(|error| EditorError::Project(error.to_string()))
}

fn editor_autosave_document_identity(
    project_root: &Path,
    source_path: PathBuf,
) -> Result<EditorAutosaveDocumentIdentity, EditorError> {
    let project_root = ProjectPaths::resolve_path(project_root)
        .map_err(|error| EditorError::Project(error.to_string()))?;
    let physical_source_path = if source_path.is_absolute() {
        ProjectPaths::resolve_path(&source_path)
    } else {
        ProjectPaths::resolve_path_from(&project_root, &source_path)
    }
    .map(|path| path.into_operation_path())
    .map_err(|error| EditorError::Project(error.to_string()))?;
    let relative_path = physical_source_path
        .strip_prefix(project_root.operation_path())
        .map_err(|_| {
            EditorError::Project(format!(
                "autosave source {} is outside project root {}",
                physical_source_path.display(),
                project_root.display_path().display()
            ))
        })?;
    let source_path = AutosaveSourcePath::parse(relative_path)
        .map_err(|error| EditorError::Project(error.to_string()))?;
    Ok(EditorAutosaveDocumentIdentity {
        document: AutosaveDocumentId::from_source_path(&source_path),
        source_path,
        physical_source_path,
    })
}

struct EditorDocumentAutosaveSource {
    manager: Weak<EditorManager>,
    document: DocumentId,
    dirty_generation: u64,
    sequence: u64,
    source_path: AutosaveSourcePath,
    physical_source_path: PathBuf,
}

impl AutosaveSnapshotSource for EditorDocumentAutosaveSource {
    fn source_path(&self) -> AutosaveSourcePath {
        self.source_path.clone()
    }

    fn capture(&self, _document: &AutosaveDocumentId) -> Result<AutosaveSnapshot, JobError> {
        let manager = self.manager.upgrade().ok_or_else(|| {
            JobError::failed(std::io::Error::other(
                "editor manager was released before autosave capture",
            ))
        })?;
        let payload = manager
            .host
            .capture_document_autosave(self.document, self.dirty_generation)
            .map_err(JobError::failed)?;
        let captured_source_path = ProjectPaths::resolve_path(payload.source_path())
            .map(|path| path.into_operation_path())
            .unwrap_or_else(|_| payload.source_path().to_path_buf());
        let extension = payload
            .source_path()
            .extension()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                JobError::failed(std::io::Error::other(format!(
                    "autosave source {} has no UTF-8 extension",
                    payload.source_path().display()
                )))
            })?;
        let extension = AutosaveExtension::parse(extension).map_err(JobError::failed)?;
        if captured_source_path != self.physical_source_path {
            return Err(JobError::failed(std::io::Error::other(
                "autosave source identity changed after admission",
            )));
        }
        let source_digest =
            AutosaveSourceDigest::observe(&self.physical_source_path).map_err(JobError::failed)?;
        Ok(AutosaveSnapshot::new(
            self.sequence,
            extension,
            self.source_path.clone(),
            AutosaveSnapshotProvenance::capture(self.dirty_generation, source_digest),
            payload.into_bytes(),
        ))
    }
}

pub(super) struct ForegroundDocumentSaveJob {
    manager: Weak<EditorManager>,
    instance_id: crate::ui::workbench::view::ViewInstanceId,
    reason: SaveReason,
}

impl ForegroundDocumentSaveJob {
    pub(super) fn new(
        manager: Weak<EditorManager>,
        instance_id: crate::ui::workbench::view::ViewInstanceId,
        reason: SaveReason,
    ) -> Self {
        Self {
            manager,
            instance_id,
            reason,
        }
    }

    pub(super) fn spec(document: DocumentId, mutex: MutexGroup) -> EditorJobSpec {
        EditorJobSpec::new(
            format!("save_document_{}", document.value()),
            JobCategory::InteractiveSave,
        )
        .with_priority(JobPriority::Interactive)
        .with_mutex_group(mutex)
        .with_estimated_bytes(std::mem::size_of::<Self>().max(1))
    }
}

impl EditorJob for ForegroundDocumentSaveJob {
    type Output = DocumentSaveReport;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        let manager = self.manager.upgrade().ok_or_else(|| {
            JobError::failed(std::io::Error::other(
                "editor manager was released before foreground document save",
            ))
        })?;
        manager
            .host
            .save_document_toolkit_canonical(&self.instance_id, self.reason)
            .map_err(JobError::failed)
    }
}

fn next_autosave_sequence() -> u64 {
    static LAST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    let wall_clock = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX))
        .unwrap_or(1);
    let mut previous = LAST_SEQUENCE.load(Ordering::Acquire);
    loop {
        let next = wall_clock.max(previous.saturating_add(1));
        match LAST_SEQUENCE.compare_exchange_weak(
            previous,
            next,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return next,
            Err(current) => previous = current,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{editor_autosave_document_identity, next_autosave_sequence};

    #[test]
    fn foreground_save_job_is_bound_to_the_editor_manager_not_the_runtime_core() {
        let source = include_str!("editor_document_autosave.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("autosave tests must remain separate from production code");
        let job = production
            .split_once("pub(super) struct ForegroundDocumentSaveJob {")
            .and_then(|(_, remainder)| remainder.split_once("impl ForegroundDocumentSaveJob"))
            .map(|(job, _)| job)
            .expect("foreground save job must remain a distinct owner");
        let run = production
            .split_once("impl EditorJob for ForegroundDocumentSaveJob {")
            .map(|(_, run)| run)
            .expect("foreground save job must retain its execution owner");

        assert!(job.contains("manager: Weak<EditorManager>"));
        assert!(run.contains("self.manager.upgrade()"));
        assert!(!job.contains("core:"));
        assert!(!production.contains("resolve_manager::<EditorManager>"));
    }

    #[test]
    fn autosave_snapshot_sequences_are_strictly_monotonic_in_process() {
        let first = next_autosave_sequence();
        let second = next_autosave_sequence();

        assert!(second > first);
    }

    #[test]
    fn autosave_document_identity_is_stable_for_a_project_relative_source() {
        let first = editor_autosave_document_identity(
            Path::new("project"),
            Path::new("assets/player.zui").to_path_buf(),
        )
        .unwrap();
        let second = editor_autosave_document_identity(
            Path::new("project"),
            Path::new("assets/player.zui").to_path_buf(),
        )
        .unwrap();

        assert_eq!(first.document, second.document);
        assert_eq!(first.source_path.as_path(), Path::new("assets/player.zui"));
    }

    #[test]
    fn autosave_document_identity_resolves_relative_sources_from_the_project_root() {
        let project_root = std::env::current_dir()
            .unwrap()
            .join("autosave-relative-project");
        let relative = editor_autosave_document_identity(
            &project_root,
            Path::new("assets/player.zui").to_path_buf(),
        )
        .unwrap();
        let rooted = editor_autosave_document_identity(
            &project_root,
            project_root.join("assets/player.zui"),
        )
        .unwrap();

        assert_eq!(relative.document, rooted.document);
        assert_eq!(
            relative.source_path.as_path(),
            Path::new("assets/player.zui")
        );
    }
}
