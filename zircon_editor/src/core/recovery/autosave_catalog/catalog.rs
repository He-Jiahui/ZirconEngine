use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::super::autosave::write_new_atomically;
use super::{
    AutosaveRecoveryCatalogDiagnostic, AutosaveRecoveryCatalogReport, AutosaveRecoveryMetadata,
    AutosaveSnapshotMetadata, AutosaveSourcePath, RECOVERY_METADATA_FILE_NAME,
    snapshot_metadata_path, snapshot_metadata_sequence,
};
use crate::core::recovery::{
    AutosaveContentDigest, AutosaveDocumentId, AutosaveError, AutosaveSourceDigest,
    RestoreCandidate, RestoreFreshness,
};

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
            Ok(metadata) => verify_source(document, source_path, &metadata),
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
                    verify_source(document, source_path, &read_metadata(&metadata_path)?)
                }
            }
            Err(error) => Err(error),
        }
    }

    pub(crate) fn recovery_catalog(&self) -> Result<AutosaveRecoveryCatalogReport, AutosaveError> {
        let entries = match fs::read_dir(&self.autosave_root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(AutosaveRecoveryCatalogReport::default());
            }
            Err(source) => {
                return Err(AutosaveError::Io {
                    operation: "read autosave recovery catalog",
                    path: self.autosave_root.clone(),
                    source,
                });
            }
        };

        let mut candidates = Vec::new();
        let mut diagnostics = Vec::new();
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(source) => {
                    diagnostics.push(AutosaveRecoveryCatalogDiagnostic::from_entry_error(
                        &self.autosave_root,
                        AutosaveError::Io {
                            operation: "enumerate autosave recovery catalog",
                            path: self.autosave_root.clone(),
                            source,
                        },
                    ));
                    continue;
                }
            };
            let directory = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(source) => {
                    diagnostics.push(AutosaveRecoveryCatalogDiagnostic::from_entry_error(
                        &directory,
                        AutosaveError::Io {
                            operation: "inspect autosave recovery catalog entry",
                            path: directory.clone(),
                            source,
                        },
                    ));
                    continue;
                }
            };
            if !file_type.is_dir() {
                continue;
            }
            match self.recovery_candidate(&directory) {
                Ok(Some(candidate)) => candidates.push(candidate),
                Ok(None) => {}
                Err(error) => diagnostics.push(
                    AutosaveRecoveryCatalogDiagnostic::from_entry_error(&directory, error),
                ),
            }
        }
        candidates.sort_by(|left, right| left.document().cmp(right.document()));
        Ok(AutosaveRecoveryCatalogReport::new(candidates, diagnostics))
    }

    fn recovery_candidate(
        &self,
        directory: &Path,
    ) -> Result<Option<RestoreCandidate>, AutosaveError> {
        let document = directory
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| AutosaveError::InvalidRecoveryDocumentDirectory {
                path: directory.to_path_buf(),
            })?;
        let document = AutosaveDocumentId::parse(document).map_err(|_| {
            AutosaveError::InvalidRecoveryDocumentDirectory {
                path: directory.to_path_buf(),
            }
        })?;
        let metadata_path = directory.join(RECOVERY_METADATA_FILE_NAME);
        let source_metadata = read_metadata(&metadata_path)?;
        let Some((sequence, snapshot_metadata)) = latest_snapshot_metadata(directory)? else {
            return Ok(None);
        };
        verify_snapshot_source(&document, &source_metadata, &snapshot_metadata)?;
        let autosave_path = self.snapshot_path(directory, sequence, snapshot_metadata.extension());
        let bytes = fs::read(&autosave_path).map_err(|source| AutosaveError::Io {
            operation: "read committed autosave snapshot",
            path: autosave_path.clone(),
            source,
        })?;
        if &AutosaveContentDigest::from_bytes(&bytes) != snapshot_metadata.committed_checksum() {
            return Err(AutosaveError::SnapshotChecksumMismatch {
                snapshot: autosave_path,
            });
        }
        let source_path = self
            .project_root
            .join(source_metadata.source_path().as_path());
        let current_source =
            AutosaveSourceDigest::observe(&source_path).map_err(|source| AutosaveError::Io {
                operation: "digest recovery source",
                path: source_path.clone(),
                source,
            })?;
        let freshness = RestoreFreshness::from_snapshot(
            snapshot_metadata.provenance().source_digest(),
            snapshot_metadata.committed_checksum(),
            &current_source,
        );
        if freshness == RestoreFreshness::SnapshotAlreadyCommitted {
            return Ok(None);
        }
        Ok(Some(RestoreCandidate::new(
            document,
            source_path,
            self.snapshot_path(directory, sequence, snapshot_metadata.extension()),
            freshness,
        )))
    }

    fn snapshot_path(
        &self,
        directory: &Path,
        sequence: u64,
        extension: &crate::core::recovery::AutosaveExtension,
    ) -> PathBuf {
        directory.join(format!("{sequence}.{}", extension.as_str()))
    }
}

fn verify_source(
    document: &AutosaveDocumentId,
    source_path: &AutosaveSourcePath,
    metadata: &AutosaveRecoveryMetadata,
) -> Result<(), AutosaveError> {
    if metadata.source_path() != source_path
        || metadata.source_identity()
            != &AutosaveContentDigest::from_bytes(
                source_path
                    .as_path()
                    .to_str()
                    .expect("autosave source paths are validated as UTF-8")
                    .as_bytes(),
            )
    {
        return Err(AutosaveError::RecoverySourceConflict {
            document: document.as_str().to_string(),
            recorded: metadata.source_path().as_path().to_path_buf(),
            requested: source_path.as_path().to_path_buf(),
        });
    }
    Ok(())
}

fn verify_snapshot_source(
    document: &AutosaveDocumentId,
    source_metadata: &AutosaveRecoveryMetadata,
    snapshot_metadata: &AutosaveSnapshotMetadata,
) -> Result<(), AutosaveError> {
    if snapshot_metadata.document() == document
        && snapshot_metadata.source_path() == source_metadata.source_path()
        && snapshot_metadata.source_identity() == source_metadata.source_identity()
    {
        return Ok(());
    }
    Err(AutosaveError::RecoverySourceConflict {
        document: document.as_str().to_string(),
        recorded: source_metadata.source_path().as_path().to_path_buf(),
        requested: snapshot_metadata.source_path().as_path().to_path_buf(),
    })
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

fn latest_snapshot_metadata(
    directory: &Path,
) -> Result<Option<(u64, AutosaveSnapshotMetadata)>, AutosaveError> {
    let entries = fs::read_dir(directory).map_err(|source| AutosaveError::Io {
        operation: "read autosave recovery document directory",
        path: directory.to_path_buf(),
        source,
    })?;
    let mut latest = None;
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
        let Some(sequence) = snapshot_metadata_sequence(&name) else {
            continue;
        };
        if latest
            .as_ref()
            .is_none_or(|(current, _)| sequence > *current)
        {
            latest = Some((sequence, entry.path()));
        }
    }
    let Some((sequence, metadata_path)) = latest else {
        return Ok(None);
    };
    let metadata = read_snapshot_metadata(&metadata_path)?;
    let expected_path = snapshot_metadata_path(directory, sequence);
    if metadata_path != expected_path {
        return Err(AutosaveError::InvalidRecoveryMetadata {
            path: metadata_path,
            message: "snapshot metadata path does not match its sequence".to_string(),
        });
    }
    Ok(Some((sequence, metadata)))
}

fn read_snapshot_metadata(path: &Path) -> Result<AutosaveSnapshotMetadata, AutosaveError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Err(AutosaveError::RecoveryMetadataMissing {
                path: path.to_path_buf(),
            });
        }
        Err(source) => {
            return Err(AutosaveError::Io {
                operation: "read autosave snapshot metadata",
                path: path.to_path_buf(),
                source,
            });
        }
    };
    AutosaveSnapshotMetadata::decode(path, &bytes)
}
