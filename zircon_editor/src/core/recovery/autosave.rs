use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use thiserror::Error;
use zircon_runtime::asset::project::ProjectPaths;

use crate::core::editing::engine::HistoryDirtyState;
use crate::core::jobs::{EditorJobSpec, JobCategory, JobPriority, MutexGroup};

use super::autosave_catalog::AutosaveRecoveryCatalog;
use super::{AutosaveSourcePath, RestoreCandidate};

pub const AUTOSAVE_RETAINED_SNAPSHOT_COUNT: usize = 3;
const AUTOSAVE_DIRECTORY: &str = "autosave";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosaveDocumentId(String);

impl AutosaveDocumentId {
    pub fn parse(value: impl Into<String>) -> Result<Self, AutosaveError> {
        let value = value.into();
        if value.is_empty()
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_uppercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'_' | b'-')
            })
        {
            return Err(AutosaveError::InvalidDocumentId { value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Derives the persistent recovery identity from the normalized project-relative source.
    pub fn from_source_path(source_path: &AutosaveSourcePath) -> Self {
        let source = source_path
            .as_path()
            .to_str()
            .expect("autosave source paths are validated as UTF-8");
        Self(format!(
            "document_{}",
            blake3::hash(source.as_bytes()).to_hex()
        ))
    }
}

impl Ord for AutosaveDocumentId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for AutosaveDocumentId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosaveExtension(String);

impl AutosaveExtension {
    pub fn parse(value: impl Into<String>) -> Result<Self, AutosaveError> {
        let value = value.into();
        if value.is_empty()
            || !value.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_uppercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'_' | b'-')
            })
        {
            return Err(AutosaveError::InvalidExtension { value });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutosavePolicy {
    interval: Duration,
}

impl AutosavePolicy {
    pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(300);

    pub fn new(interval: Duration) -> Result<Self, AutosaveError> {
        if interval.is_zero() {
            return Err(AutosaveError::ZeroInterval);
        }
        Ok(Self { interval })
    }

    pub const fn interval(self) -> Duration {
        self.interval
    }
}

impl Default for AutosavePolicy {
    fn default() -> Self {
        Self {
            interval: Self::DEFAULT_INTERVAL,
        }
    }
}

/// Immutable scheduling constraints for an autosave task.
///
/// The save owner supplies its mutex group so autosaves can never overlap a
/// foreground save of the same document. The recovery core deliberately does
/// not submit jobs itself; Editor14 owns queue admission and execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosaveJobPolicy {
    save_mutex_group: MutexGroup,
}

impl AutosaveJobPolicy {
    pub fn for_save_mutex(save_mutex_group: MutexGroup) -> Self {
        Self { save_mutex_group }
    }

    pub const fn category(&self) -> JobCategory {
        JobCategory::Misc
    }

    pub const fn priority(&self) -> JobPriority {
        JobPriority::Background
    }

    pub fn save_mutex_group(&self) -> &MutexGroup {
        &self.save_mutex_group
    }

    pub fn build_job_spec(&self, document: &AutosaveDocumentId) -> EditorJobSpec {
        EditorJobSpec::new(format!("autosave:{}", document.as_str()), self.category())
            .with_priority(self.priority())
            .with_mutex_group(self.save_mutex_group.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosaveDocumentState {
    document: AutosaveDocumentId,
    dirty: bool,
}

impl AutosaveDocumentState {
    /// Projects Editor03's saved-top dirty fact for one document.
    ///
    /// Document-to-history routing belongs to the editor manager. Autosave
    /// receives only this immutable query result and never owns dirty state.
    pub fn from_history_dirty(document: AutosaveDocumentId, state: HistoryDirtyState) -> Self {
        Self {
            document,
            dirty: state.is_dirty(),
        }
    }

    pub(crate) const fn from_dirty_projection(document: AutosaveDocumentId, dirty: bool) -> Self {
        Self { document, dirty }
    }

    #[cfg(test)]
    pub(crate) fn from_dirty_for_test(document: AutosaveDocumentId, dirty: bool) -> Self {
        Self { document, dirty }
    }

    pub fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }

    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosavePlan {
    documents: Vec<AutosaveDocumentId>,
}

impl AutosavePlan {
    pub fn documents(&self) -> &[AutosaveDocumentId] {
        &self.documents
    }
}

#[derive(Clone, Debug)]
pub struct AutosaveScheduler {
    policy: AutosavePolicy,
    next_due_at: Duration,
    in_flight: bool,
}

impl AutosaveScheduler {
    pub fn new(policy: AutosavePolicy) -> Self {
        Self {
            next_due_at: policy.interval(),
            policy,
            in_flight: false,
        }
    }

    pub fn plan(
        &mut self,
        now: Duration,
        documents: &[AutosaveDocumentState],
    ) -> Option<AutosavePlan> {
        self.plan_window(now, documents, usize::MAX, None)
    }

    pub(super) fn is_due(&self, now: Duration) -> bool {
        !self.in_flight && now >= self.next_due_at
    }

    pub(super) fn plan_window(
        &mut self,
        now: Duration,
        documents: &[AutosaveDocumentState],
        max_documents: usize,
        start_after: Option<&AutosaveDocumentId>,
    ) -> Option<AutosavePlan> {
        if !self.is_due(now) || max_documents == 0 {
            return None;
        }
        let mut after_cursor = BTreeSet::new();
        let mut wrapped = BTreeSet::new();
        for state in documents.iter().filter(|state| state.is_dirty()) {
            let document = state.document();
            if start_after.is_some_and(|cursor| document <= cursor) {
                insert_bounded_document(&mut wrapped, document.clone(), max_documents);
            } else {
                insert_bounded_document(&mut after_cursor, document.clone(), max_documents);
            }
        }
        let mut documents = after_cursor.into_iter().collect::<Vec<_>>();
        documents.extend(
            wrapped
                .into_iter()
                .take(max_documents.saturating_sub(documents.len())),
        );
        if documents.is_empty() {
            return None;
        }
        self.in_flight = true;
        Some(AutosavePlan { documents })
    }

    /// Completes an admitted autosave job, regardless of its write result.
    pub fn mark_finished(&mut self, at: Duration) {
        self.in_flight = false;
        self.next_due_at = at
            .checked_add(self.policy.interval())
            .unwrap_or(Duration::MAX);
    }

    /// Releases a plan whose job was not admitted, so the caller can retry it.
    pub fn mark_submission_failed(&mut self) {
        self.in_flight = false;
    }

    pub const fn is_in_flight(&self) -> bool {
        self.in_flight
    }
}

fn insert_bounded_document(
    documents: &mut BTreeSet<AutosaveDocumentId>,
    document: AutosaveDocumentId,
    max_documents: usize,
) {
    documents.insert(document);
    if documents.len() > max_documents {
        documents.pop_last();
    }
}

#[derive(Clone, Debug)]
pub struct AutosaveStore {
    project_root: PathBuf,
}

impl AutosaveStore {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        Self {
            project_root: ProjectPaths::resolve_path(&project_root)
                .map(|root| root.into_operation_path())
                .unwrap_or(project_root),
        }
    }

    pub fn autosave_root(&self) -> PathBuf {
        self.project_root.join(".zircon").join(AUTOSAVE_DIRECTORY)
    }

    pub fn snapshot_path(
        &self,
        document: &AutosaveDocumentId,
        sequence: u64,
        extension: &AutosaveExtension,
    ) -> Result<PathBuf, AutosaveError> {
        if sequence == 0 {
            return Err(AutosaveError::InvalidSequence { sequence });
        }
        Ok(self
            .autosave_root()
            .join(document.as_str())
            .join(format!("{sequence}.{}", extension.as_str())))
    }

    pub fn write_snapshot(
        &self,
        document: &AutosaveDocumentId,
        source_path: &AutosaveSourcePath,
        sequence: u64,
        extension: &AutosaveExtension,
        bytes: &[u8],
    ) -> Result<PathBuf, AutosaveError> {
        let path = self.snapshot_path(document, sequence, extension)?;
        let directory = path
            .parent()
            .expect("autosave snapshot path always has a document directory");
        fs::create_dir_all(directory).map_err(|source| AutosaveError::Io {
            operation: "create autosave directory",
            path: directory.to_path_buf(),
            source,
        })?;
        let _sequence_reservation = self.reserve_sequence(directory, sequence, &path)?;
        AutosaveRecoveryCatalog::new(&self.project_root, self.autosave_root()).persist_source(
            document,
            directory,
            source_path,
        )?;
        write_atomically(&path, bytes, "write autosave snapshot")?;
        self.rotate_document(document)
            .map_err(|source| AutosaveError::RotationAfterWrite {
                snapshot: path.clone(),
                source: Box::new(source),
            })?;
        Ok(path)
    }

    pub(crate) fn next_sequence(
        &self,
        document: &AutosaveDocumentId,
        proposed: u64,
    ) -> Result<u64, AutosaveError> {
        if proposed == 0 {
            return Err(AutosaveError::InvalidSequence { sequence: proposed });
        }
        let directory = self.autosave_root().join(document.as_str());
        let persisted = match latest_occupied_sequence(&directory) {
            Ok(sequence) => sequence,
            Err(AutosaveError::Io { source, .. }) if source.kind() == io::ErrorKind::NotFound => {
                None
            }
            Err(error) => return Err(error),
        };
        match persisted {
            Some(sequence) => sequence
                .checked_add(1)
                .map(|next| proposed.max(next))
                .ok_or_else(|| AutosaveError::SequenceExhausted {
                    document: document.as_str().to_string(),
                }),
            None => Ok(proposed),
        }
    }

    /// Rebuilds recovery candidates only from snapshot metadata written with the autosave.
    pub fn recovery_candidates(&self) -> Result<Vec<RestoreCandidate>, AutosaveError> {
        AutosaveRecoveryCatalog::new(&self.project_root, self.autosave_root()).recovery_candidates()
    }

    fn reserve_sequence(
        &self,
        directory: &Path,
        sequence: u64,
        snapshot_path: &Path,
    ) -> Result<AutosaveSequenceReservation, AutosaveError> {
        if snapshot_sequence_exists(directory, sequence)? {
            return Err(AutosaveError::SnapshotSequenceUnavailable {
                directory: directory.to_path_buf(),
                sequence,
            });
        }

        let path = sequence_reservation_path(directory, sequence);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => drop(file),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
                return Err(AutosaveError::SnapshotSequenceUnavailable {
                    directory: directory.to_path_buf(),
                    sequence,
                });
            }
            Err(source) => {
                return Err(AutosaveError::Io {
                    operation: "create autosave sequence reservation",
                    path,
                    source,
                });
            }
        }
        let reservation = AutosaveSequenceReservation { path };

        if snapshot_path.exists() {
            return Err(AutosaveError::SnapshotAlreadyExists {
                path: snapshot_path.to_path_buf(),
            });
        }
        Ok(reservation)
    }

    fn rotate_document(&self, document: &AutosaveDocumentId) -> Result<(), AutosaveError> {
        let directory = self.autosave_root().join(document.as_str());
        let entries = fs::read_dir(&directory).map_err(|source| AutosaveError::Io {
            operation: "read autosave directory",
            path: directory.clone(),
            source,
        })?;
        let mut snapshots = BTreeMap::<u64, Vec<PathBuf>>::new();
        for entry in entries {
            let entry = entry.map_err(|source| AutosaveError::Io {
                operation: "enumerate autosave directory",
                path: directory.clone(),
                source,
            })?;
            let file_type = entry.file_type().map_err(|source| AutosaveError::Io {
                operation: "inspect autosave entry",
                path: entry.path(),
                source,
            })?;
            if !file_type.is_file() {
                continue;
            }
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let Some((sequence, _)) = name.split_once('.') else {
                continue;
            };
            let Ok(sequence) = sequence.parse::<u64>() else {
                continue;
            };
            if sequence == 0 {
                continue;
            }
            snapshots.entry(sequence).or_default().push(entry.path());
        }

        while snapshots.len() > AUTOSAVE_RETAINED_SNAPSHOT_COUNT {
            let oldest = *snapshots
                .keys()
                .next()
                .expect("non-empty autosave sequence map has a first sequence");
            for path in snapshots
                .remove(&oldest)
                .expect("oldest autosave sequence remains present")
            {
                fs::remove_file(&path).map_err(|source| AutosaveError::Io {
                    operation: "rotate autosave snapshot",
                    path,
                    source,
                })?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum AutosaveError {
    #[error("autosave document id `{value}` must use only ASCII letters, digits, `_`, or `-`")]
    InvalidDocumentId { value: String },
    #[error("autosave extension `{value}` must use only ASCII letters, digits, `_`, or `-`")]
    InvalidExtension { value: String },
    #[error("autosave interval must be greater than zero")]
    ZeroInterval,
    #[error("autosave sequence must be greater than zero, received {sequence}")]
    InvalidSequence { sequence: u64 },
    #[error("autosave sequence space is exhausted for document `{document}`")]
    SequenceExhausted { document: String },
    #[error(
        "autosave recovery source path `{path}` must be non-empty, UTF-8, and project-relative"
    )]
    InvalidRecoverySourcePath { path: PathBuf },
    #[error(
        "autosave document `{document}` is already mapped to `{recorded}` and cannot be remapped to `{requested}`"
    )]
    RecoverySourceConflict {
        document: String,
        recorded: PathBuf,
        requested: PathBuf,
    },
    #[error("autosave recovery metadata is missing at `{path}`")]
    RecoveryMetadataMissing { path: PathBuf },
    #[error("autosave recovery metadata at `{path}` is invalid: {message}")]
    InvalidRecoveryMetadata { path: PathBuf, message: String },
    #[error("autosave recovery directory `{path}` is not a valid document identifier")]
    InvalidRecoveryDocumentDirectory { path: PathBuf },
    #[error(
        "autosave recovery directory `{directory}` has multiple snapshots for sequence {sequence}"
    )]
    DuplicateRecoverySequence { directory: PathBuf, sequence: u64 },
    #[error("autosave snapshot already exists at `{path}`")]
    SnapshotAlreadyExists { path: PathBuf },
    #[error("autosave snapshot sequence {sequence} is already in use for `{directory}`")]
    SnapshotSequenceUnavailable { directory: PathBuf, sequence: u64 },
    #[error(
        "autosave snapshot `{snapshot}` was persisted, but retention rotation failed: {source}"
    )]
    RotationAfterWrite {
        snapshot: PathBuf,
        #[source]
        source: Box<AutosaveError>,
    },
    #[error("failed to {operation} `{path}`: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

struct AutosaveSequenceReservation {
    path: PathBuf,
}

impl Drop for AutosaveSequenceReservation {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn sequence_reservation_path(directory: &Path, sequence: u64) -> PathBuf {
    directory.join(format!(".{sequence}.autosave-reservation"))
}

fn snapshot_sequence_exists(directory: &Path, sequence: u64) -> Result<bool, AutosaveError> {
    let entries = fs::read_dir(directory).map_err(|source| AutosaveError::Io {
        operation: "read autosave directory",
        path: directory.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| AutosaveError::Io {
            operation: "enumerate autosave directory",
            path: directory.to_path_buf(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| AutosaveError::Io {
            operation: "inspect autosave entry",
            path: entry.path(),
            source,
        })?;
        if !file_type.is_file() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Some((candidate, _)) = name.split_once('.') else {
            continue;
        };
        if candidate.parse::<u64>().ok() == Some(sequence) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn latest_occupied_sequence(directory: &Path) -> Result<Option<u64>, AutosaveError> {
    let entries = fs::read_dir(directory).map_err(|source| AutosaveError::Io {
        operation: "read autosave directory",
        path: directory.to_path_buf(),
        source,
    })?;
    let mut latest = None;
    for entry in entries {
        let entry = entry.map_err(|source| AutosaveError::Io {
            operation: "enumerate autosave directory",
            path: directory.to_path_buf(),
            source,
        })?;
        if !entry
            .file_type()
            .map_err(|source| AutosaveError::Io {
                operation: "inspect autosave entry",
                path: entry.path(),
                source,
            })?
            .is_file()
        {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let sequence = name
            .strip_prefix('.')
            .and_then(|name| name.strip_suffix(".autosave-reservation"))
            .or_else(|| name.split_once('.').map(|(sequence, _)| sequence));
        let Some(Ok(sequence)) = sequence.map(str::parse::<u64>) else {
            continue;
        };
        if sequence != 0 {
            latest = Some(latest.map_or(sequence, |current: u64| current.max(sequence)));
        }
    }
    Ok(latest)
}

pub(super) fn write_atomically(
    path: &Path,
    bytes: &[u8],
    operation: &'static str,
) -> Result<(), AutosaveError> {
    let (temporary, mut file) = create_temporary_file(path)?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)?;
        sync_parent_directory(path)?;
        Ok(())
    })();
    if let Err(source) = result {
        let _ = fs::remove_file(&temporary);
        return Err(AutosaveError::Io {
            operation,
            path: path.to_path_buf(),
            source,
        });
    }
    Ok(())
}

/// Publishes immutable recovery metadata without replacing a concurrent source mapping.
pub(super) fn write_new_atomically(
    path: &Path,
    bytes: &[u8],
    operation: &'static str,
) -> Result<bool, AutosaveError> {
    let (temporary, mut file) = create_temporary_file(path)?;
    let result = (|| -> io::Result<bool> {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        match fs::hard_link(&temporary, path) {
            Ok(()) => {
                let _ = fs::remove_file(&temporary);
                sync_parent_directory(path)?;
                Ok(true)
            }
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
                let _ = fs::remove_file(&temporary);
                Ok(false)
            }
            Err(source) => Err(source),
        }
    })();
    match result {
        Ok(created) => Ok(created),
        Err(source) => {
            let _ = fs::remove_file(&temporary);
            Err(AutosaveError::Io {
                operation,
                path: path.to_path_buf(),
                source,
            })
        }
    }
}

fn create_temporary_file(path: &Path) -> Result<(PathBuf, fs::File), AutosaveError> {
    let parent = path.parent().expect("autosave snapshot path has a parent");
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("autosave snapshot path has a UTF-8 file name");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for attempt in 0_u8..32 {
        let temporary = parent.join(format!(
            ".{file_name}.{}.{}.{}.tmp",
            std::process::id(),
            nonce,
            attempt
        ));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => return Ok((temporary, file)),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(source) => {
                return Err(AutosaveError::Io {
                    operation: "create autosave temporary file",
                    path: temporary,
                    source,
                });
            }
        }
    }
    Err(AutosaveError::Io {
        operation: "allocate autosave temporary file",
        path: path.to_path_buf(),
        source: io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not allocate a unique autosave temporary file",
        ),
    })
}

#[cfg(not(windows))]
fn sync_parent_directory(path: &Path) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::File::open(parent)?.sync_all()
}

#[cfg(windows)]
fn sync_parent_directory(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use zircon_runtime::asset::project::ProjectPaths;

    use super::{AutosaveDocumentId, AutosaveExtension, AutosaveSourcePath, AutosaveStore};

    #[test]
    fn persisted_sequence_advances_a_lower_post_restart_proposal() {
        let project = unique_autosave_root("sequence-restart");
        let store = AutosaveStore::new(&project);
        let source = AutosaveSourcePath::parse("assets/player.zui").unwrap();
        let document = AutosaveDocumentId::from_source_path(&source);
        let extension = AutosaveExtension::parse("zui").unwrap();
        let first_sequence = store.next_sequence(&document, 100).unwrap();
        let first = store
            .write_snapshot(&document, &source, first_sequence, &extension, b"first")
            .unwrap();
        let second_sequence = store.next_sequence(&document, 1).unwrap();
        let second = store
            .write_snapshot(&document, &source, second_sequence, &extension, b"second")
            .unwrap();
        assert!(first.ends_with("100.zui"));
        assert!(second.ends_with("101.zui"));
        fs::remove_dir_all(project).unwrap();
    }

    #[test]
    fn document_id_is_stable_for_a_normalized_project_relative_source() {
        let first = AutosaveSourcePath::parse("assets/ui/panel.zui").unwrap();
        let same = AutosaveSourcePath::parse("assets/ui/panel.zui").unwrap();
        let other = AutosaveSourcePath::parse("assets/ui/other.zui").unwrap();

        assert_eq!(
            AutosaveDocumentId::from_source_path(&first),
            AutosaveDocumentId::from_source_path(&same)
        );
        assert_ne!(
            AutosaveDocumentId::from_source_path(&first),
            AutosaveDocumentId::from_source_path(&other)
        );
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn autosave_root_keeps_the_physical_project_identity() {
        let parent = unique_autosave_root("physical-identity");
        let physical_project = parent.join("physical-project");
        fs::create_dir_all(&physical_project).unwrap();
        let project_alias = parent.join("project-alias");
        create_directory_link(&physical_project, &project_alias);

        let autosave_root = AutosaveStore::new(&project_alias).autosave_root();
        let expected = ProjectPaths::resolve_existing_path(&physical_project)
            .unwrap()
            .join(".zircon/autosave");

        fs::remove_dir_all(&parent).unwrap();
        assert_eq!(autosave_root, expected);
    }

    fn unique_autosave_root(case_name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "zircon-editor-autosave-store-{case_name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[cfg(unix)]
    fn create_directory_link(target: &Path, link: &Path) {
        std::os::unix::fs::symlink(target, link).expect("create autosave project alias");
    }

    #[cfg(windows)]
    fn create_directory_link(target: &Path, link: &Path) {
        let command = format!(r#"mklink /J "{}" "{}""#, link.display(), target.display());
        let output = std::process::Command::new("cmd")
            .args(["/D", "/S", "/C"])
            .arg(command)
            .output()
            .expect("start mklink for autosave project alias");
        assert!(
            output.status.success(),
            "create autosave project junction failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
