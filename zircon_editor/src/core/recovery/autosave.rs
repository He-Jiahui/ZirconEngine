use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use thiserror::Error;

use crate::core::editing::engine::HistoryDirtyState;
use crate::core::jobs::{EditorJobSpec, JobCategory, JobPriority, MutexGroup};

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
        if self.in_flight || now < self.next_due_at {
            return None;
        }
        let documents = documents
            .iter()
            .filter(|state| state.is_dirty())
            .map(|state| state.document().clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
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

#[derive(Clone, Debug)]
pub struct AutosaveStore {
    project_root: PathBuf,
    reserved_sequences: Arc<Mutex<BTreeSet<(AutosaveDocumentId, u64)>>>,
}

impl AutosaveStore {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
            reserved_sequences: Arc::default(),
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
        let _sequence_reservation = self.reserve_sequence(document, directory, sequence, &path)?;
        write_atomically(&path, bytes)?;
        self.rotate_document(document)
            .map_err(|source| AutosaveError::RotationAfterWrite {
                snapshot: path.clone(),
                source: Box::new(source),
            })?;
        Ok(path)
    }

    fn reserve_sequence(
        &self,
        document: &AutosaveDocumentId,
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

        let key = (document.clone(), sequence);
        let mut reserved = self
            .reserved_sequences
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !reserved.insert(key.clone()) {
            return Err(AutosaveError::SnapshotSequenceUnavailable {
                directory: directory.to_path_buf(),
                sequence,
            });
        }
        drop(reserved);

        if snapshot_path.exists() {
            let mut reserved = self
                .reserved_sequences
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            reserved.remove(&key);
            return Err(AutosaveError::SnapshotAlreadyExists {
                path: snapshot_path.to_path_buf(),
            });
        }
        Ok(AutosaveSequenceReservation {
            reserved_sequences: Arc::clone(&self.reserved_sequences),
            key,
        })
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
    reserved_sequences: Arc<Mutex<BTreeSet<(AutosaveDocumentId, u64)>>>,
    key: (AutosaveDocumentId, u64),
}

impl Drop for AutosaveSequenceReservation {
    fn drop(&mut self) {
        let mut reserved = self
            .reserved_sequences
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        reserved.remove(&self.key);
    }
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

fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), AutosaveError> {
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
            operation: "write autosave snapshot",
            path: path.to_path_buf(),
            source,
        });
    }
    Ok(())
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
