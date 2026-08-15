use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::super::autosave::write_new_atomically;
use super::{AutosaveRecoveryMetadata, AutosaveSourcePath, RECOVERY_METADATA_FILE_NAME};
use crate::core::recovery::{AutosaveDocumentId, AutosaveError, RestoreCandidate};

pub(crate) struct AutosaveRecoveryCatalog {
    project_root: PathBuf,
    autosave_root: PathBuf,
}

impl AutosaveRecoveryCatalog {
    pub(crate) fn new(project_root: impl Into<PathBuf>, autosave_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
            autosave_root: autosave_root.into(),
        }
    }

    pub(crate) fn persist_source(
        &self,
        document: &AutosaveDocumentId,
        directory: &Path,
        source_path: &AutosaveSourcePath,
    ) -> Result<(), AutosaveError> {
        let metadata_path = directory.join(RECOVERY_METADATA_FILE_NAME);
        match read_metadata(&metadata_path) {
            Ok(metadata) => verify_source(document, source_path, metadata),
            Err(AutosaveError::RecoveryMetadataMissing { .. }) => {
                let metadata = AutosaveRecoveryMetadata::from_source_path(source_path.clone());
                let bytes = metadata.encode(&metadata_path)?;
                if write_new_atomically(
                    &metadata_path,
                    &bytes,
                    "publish autosave recovery metadata",
                )? {
                    Ok(())
                } else {
                    verify_source(document, source_path, read_metadata(&metadata_path)?)
                }
            }
            Err(error) => Err(error),
        }
    }

    pub(crate) fn recovery_candidates(&self) -> Result<Vec<RestoreCandidate>, AutosaveError> {
        let entries = match fs::read_dir(&self.autosave_root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(source) => {
                return Err(AutosaveError::Io {
                    operation: "read autosave recovery catalog",
                    path: self.autosave_root.clone(),
                    source,
                });
            }
        };

        let mut candidates = BTreeMap::new();
        for entry in entries {
            let entry = entry.map_err(|source| AutosaveError::Io {
                operation: "enumerate autosave recovery catalog",
                path: self.autosave_root.clone(),
                source,
            })?;
            let directory = entry.path();
            let file_type = entry.file_type().map_err(|source| AutosaveError::Io {
                operation: "inspect autosave recovery catalog entry",
                path: directory.clone(),
                source,
            })?;
            if !file_type.is_dir() {
                continue;
            }
            let document = directory
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| AutosaveError::InvalidRecoveryDocumentDirectory {
                    path: directory.clone(),
                })?;
            let document = AutosaveDocumentId::parse(document).map_err(|_| {
                AutosaveError::InvalidRecoveryDocumentDirectory {
                    path: directory.clone(),
                }
            })?;
            let Some(autosave_path) = latest_snapshot_path(&directory)? else {
                continue;
            };
            let metadata_path = directory.join(RECOVERY_METADATA_FILE_NAME);
            let metadata = read_metadata(&metadata_path)?;
            let source_path = self.project_root.join(metadata.source_path().as_path());
            let source_modified_at =
                modified_at_or_missing(&source_path, "inspect recovery source")?;
            let autosave_modified_at = modified_at(&autosave_path, "inspect autosave snapshot")?;
            candidates.insert(
                document.clone(),
                RestoreCandidate::new(
                    document,
                    source_path,
                    autosave_path,
                    source_modified_at,
                    autosave_modified_at,
                ),
            );
        }
        Ok(candidates.into_values().collect())
    }
}

fn verify_source(
    document: &AutosaveDocumentId,
    source_path: &AutosaveSourcePath,
    metadata: AutosaveRecoveryMetadata,
) -> Result<(), AutosaveError> {
    if metadata.source_path() != source_path {
        return Err(AutosaveError::RecoverySourceConflict {
            document: document.as_str().to_string(),
            recorded: metadata.source_path().as_path().to_path_buf(),
            requested: source_path.as_path().to_path_buf(),
        });
    }
    Ok(())
}

fn read_metadata(path: &Path) -> Result<AutosaveRecoveryMetadata, AutosaveError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Err(AutosaveError::RecoveryMetadataMissing {
                path: path.to_path_buf(),
            });
        }
        Err(source) => {
            return Err(AutosaveError::Io {
                operation: "read autosave recovery metadata",
                path: path.to_path_buf(),
                source,
            });
        }
    };
    AutosaveRecoveryMetadata::decode(path, &bytes)
}

fn latest_snapshot_path(directory: &Path) -> Result<Option<PathBuf>, AutosaveError> {
    let entries = fs::read_dir(directory).map_err(|source| AutosaveError::Io {
        operation: "read autosave recovery document directory",
        path: directory.to_path_buf(),
        source,
    })?;
    let mut snapshots = BTreeMap::new();
    for entry in entries {
        let entry = entry.map_err(|source| AutosaveError::Io {
            operation: "enumerate autosave recovery snapshots",
            path: directory.to_path_buf(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| AutosaveError::Io {
            operation: "inspect autosave recovery snapshot",
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
        if snapshots.insert(sequence, entry.path()).is_some() {
            return Err(AutosaveError::DuplicateRecoverySequence {
                directory: directory.to_path_buf(),
                sequence,
            });
        }
    }
    Ok(snapshots.into_iter().next_back().map(|(_, path)| path))
}

fn modified_at(path: &Path, operation: &'static str) -> Result<SystemTime, AutosaveError> {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .map_err(|source| AutosaveError::Io {
            operation,
            path: path.to_path_buf(),
            source,
        })
}

fn modified_at_or_missing(
    path: &Path,
    operation: &'static str,
) -> Result<Option<SystemTime>, AutosaveError> {
    match fs::metadata(path).and_then(|metadata| metadata.modified()) {
        Ok(modified_at) => Ok(Some(modified_at)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(AutosaveError::Io {
            operation,
            path: path.to_path_buf(),
            source,
        }),
    }
}
