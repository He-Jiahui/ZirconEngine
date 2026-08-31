use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::ProjectPaths;

use super::{AutosaveDocumentId, AutosaveError, AutosaveExtension};
use crate::core::recovery::autosave_catalog::{
    AutosaveRecoveryCatalog, AutosaveRecoveryCatalogReport, AutosaveSnapshotMetadata,
    AutosaveSourcePath, snapshot_metadata_path, snapshot_metadata_sequence,
};
use crate::core::recovery::{
    AutosaveContentDigest, AutosaveDiagnosticError, AutosaveDiagnosticRecord,
    AutosaveDiagnosticStore, AutosaveDocumentOutcome, AutosaveSnapshotProvenance,
};

pub const AUTOSAVE_RETAINED_SNAPSHOT_COUNT: usize = 3;
const AUTOSAVE_DIRECTORY: &str = "autosave";

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

    /// Publishes one terminal outcome from the autosave worker. The adapter
    /// never performs this normal-path persistence while polling the UI frame.
    pub(crate) fn persist_diagnostic(
        &self,
        outcome: &AutosaveDocumentOutcome,
    ) -> Result<AutosaveDiagnosticRecord, AutosaveDiagnosticError> {
        AutosaveDiagnosticStore::from_autosave_root(self.autosave_root()).append(outcome)
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
        provenance: &AutosaveSnapshotProvenance,
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
        let metadata_path = snapshot_metadata_path(directory, sequence);
        let metadata = AutosaveSnapshotMetadata::new(
            document.clone(),
            source_path.clone(),
            extension.clone(),
            provenance.clone(),
            AutosaveContentDigest::from_bytes(bytes),
        );
        let metadata_bytes = metadata.encode(&metadata_path)?;
        if !write_new_atomically(&path, bytes, "publish autosave snapshot")? {
            return Err(AutosaveError::SnapshotAlreadyExists { path });
        }
        let metadata_published = write_new_atomically(
            &metadata_path,
            &metadata_bytes,
            "publish autosave snapshot metadata",
        );
        match metadata_published {
            Ok(true) => {}
            Ok(false) => {
                let _ = fs::remove_file(&path);
                return Err(AutosaveError::SnapshotAlreadyExists {
                    path: metadata_path,
                });
            }
            Err(error) => {
                let _ = fs::remove_file(&path);
                return Err(error);
            }
        }
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

    /// Rebuilds recovery candidates and quarantines malformed document entries into diagnostics.
    ///
    /// An inaccessible catalog root remains a global error; once it is enumerable, one bad
    /// recovery directory must not suppress valid candidates from other documents.
    pub fn recovery_catalog(&self) -> Result<AutosaveRecoveryCatalogReport, AutosaveError> {
        AutosaveRecoveryCatalog::new(&self.project_root, self.autosave_root()).recovery_catalog()
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
        let mut committed_sequences = BTreeMap::<u64, ()>::new();
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
            let Some(sequence) = snapshot_metadata_sequence(&name) else {
                continue;
            };
            committed_sequences.insert(sequence, ());
        }

        while committed_sequences.len() > AUTOSAVE_RETAINED_SNAPSHOT_COUNT {
            let oldest = *committed_sequences
                .keys()
                .next()
                .expect("non-empty autosave sequence map has a first sequence");
            let sequence_prefix = format!("{oldest}.");
            for entry in fs::read_dir(&directory).map_err(|source| AutosaveError::Io {
                operation: "read autosave directory for rotation",
                path: directory.clone(),
                source,
            })? {
                let entry = entry.map_err(|source| AutosaveError::Io {
                    operation: "enumerate autosave directory for rotation",
                    path: directory.clone(),
                    source,
                })?;
                let path = entry.path();
                let is_snapshot_artifact = entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(&sequence_prefix));
                if !is_snapshot_artifact {
                    continue;
                }
                fs::remove_file(&path).map_err(|source| AutosaveError::Io {
                    operation: "rotate autosave snapshot",
                    path,
                    source,
                })?;
            }
            committed_sequences.remove(&oldest);
        }
        Ok(())
    }
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

/// Publishes an immutable recovery artifact without replacing a concurrent writer.
pub(in crate::core::recovery) fn write_new_atomically(
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
