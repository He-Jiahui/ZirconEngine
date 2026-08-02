use std::collections::HashMap;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveArtifact, RuntimeSessionArchiveError, archive_save,
};
use super::support::{ensure_parent_dir, temporary_archive_path};

static COMMITTED_PATH_REVISIONS: OnceLock<Mutex<HashMap<String, CommittedPathRevision>>> =
    OnceLock::new();

#[derive(Clone, Copy, Default)]
struct CommittedPathRevision {
    commit: u64,
    next_write_generation: u64,
    archive_lineage: u64,
    archive_revision: u64,
}

pub(super) struct ArchivePathWriteTicket {
    identity: String,
    expected_commit: u64,
    write_generation: u64,
}

impl ArchivePathWriteTicket {
    pub(super) fn write_generation(&self) -> u64 {
        self.write_generation
    }
}

pub(in crate::scene::dynamic_scene::session) fn save_to_path_atomically(
    archive: &RuntimeSessionArchive,
    path: impl AsRef<Path>,
) -> Result<(), RuntimeSessionArchiveError> {
    save_artifact_to_path_atomically(&archive.sealed_artifact()?, path)
}

pub(in crate::scene::dynamic_scene::session) fn save_artifact_to_path_atomically(
    artifact: &RuntimeSessionArchiveArtifact,
    path: impl AsRef<Path>,
) -> Result<(), RuntimeSessionArchiveError> {
    let path = canonical_archive_target(path.as_ref())?;
    let ticket = prepare_archive_path_write(artifact, &path)?;
    save_artifact_to_prepared_path_atomically(artifact, &path, ticket)
}

pub(super) fn save_artifact_to_prepared_path_atomically(
    artifact: &RuntimeSessionArchiveArtifact,
    path: &Path,
    ticket: ArchivePathWriteTicket,
) -> Result<(), RuntimeSessionArchiveError> {
    archive_save::preview_artifact_save_to_path(artifact, &path)?;
    let temp_path = temporary_archive_path(&path, "tmp");
    let write_result = (|| -> Result<(), RuntimeSessionArchiveError> {
        let file = fs::File::create(&temp_path)?;
        let mut writer = BufWriter::with_capacity(ARCHIVE_WRITE_BUFFER_BYTES, file);
        artifact.write_to(&mut writer)?;
        writer.flush()?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }

    let mut committed = committed_path_revisions()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let current = committed.get(&ticket.identity).copied().unwrap_or_default();
    if current.next_write_generation != ticket.write_generation {
        let _ = fs::remove_file(&temp_path);
        return Err(RuntimeSessionArchiveError::StalePathWrite {
            write_generation: ticket.write_generation,
            current_generation: current.next_write_generation,
        });
    }
    if current.commit != ticket.expected_commit {
        let _ = fs::remove_file(&temp_path);
        return Err(RuntimeSessionArchiveError::StalePathCommit {
            expected_commit: ticket.expected_commit,
            committed_commit: current.commit,
        });
    }
    if let Err(error) = reject_stale_lineage_revision(artifact, current) {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }

    let backup_path = prepare_existing_target_backup(&path, &temp_path)?;
    match fs::rename(&temp_path, &path) {
        Ok(()) => {
            if let Some(backup_path) = backup_path.as_ref() {
                let _ = fs::remove_file(backup_path);
            }
            committed.insert(
                ticket.identity,
                CommittedPathRevision {
                    commit: current.commit.saturating_add(1),
                    next_write_generation: current.next_write_generation,
                    archive_lineage: artifact.lineage(),
                    archive_revision: artifact.revision(),
                },
            );
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            restore_existing_target_backup(&path, backup_path.as_deref());
            Err(error.into())
        }
    }
}

const ARCHIVE_WRITE_BUFFER_BYTES: usize = 64 * 1024;

fn committed_path_revisions() -> &'static Mutex<HashMap<String, CommittedPathRevision>> {
    COMMITTED_PATH_REVISIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn prepare_archive_path_write(
    artifact: &RuntimeSessionArchiveArtifact,
    path: &Path,
) -> Result<ArchivePathWriteTicket, RuntimeSessionArchiveError> {
    let identity = archive_path_identity(path);
    let mut committed = committed_path_revisions()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let current = committed.get(&identity).copied().unwrap_or_default();
    reject_stale_lineage_revision(artifact, current)?;
    let revision = committed.entry(identity.clone()).or_default();
    revision.next_write_generation = revision.next_write_generation.saturating_add(1);
    Ok(ArchivePathWriteTicket {
        identity,
        expected_commit: current.commit,
        write_generation: revision.next_write_generation,
    })
}

fn reject_stale_lineage_revision(
    artifact: &RuntimeSessionArchiveArtifact,
    committed: CommittedPathRevision,
) -> Result<(), RuntimeSessionArchiveError> {
    if committed.archive_lineage == artifact.lineage()
        && artifact.revision() < committed.archive_revision
    {
        return Err(RuntimeSessionArchiveError::StaleArtifactRevision {
            artifact_revision: artifact.revision(),
            committed_revision: committed.archive_revision,
        });
    }
    Ok(())
}

pub(super) fn canonical_archive_target(path: &Path) -> Result<PathBuf, RuntimeSessionArchiveError> {
    ensure_parent_dir(path)?;
    if path.exists() {
        return Ok(fs::canonicalize(path)?);
    }

    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "runtime session archive target path has no file name",
        )
    })?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    let canonical_parent = fs::canonicalize(parent.unwrap_or_else(|| Path::new(".")))?;
    Ok(canonical_parent.join(file_name))
}

pub(super) fn archive_path_identity(path: &Path) -> String {
    let identity = path.to_string_lossy();
    if cfg!(windows) {
        identity.to_lowercase()
    } else {
        identity.into_owned()
    }
}

fn prepare_existing_target_backup(
    path: &Path,
    temp_path: &Path,
) -> Result<Option<PathBuf>, RuntimeSessionArchiveError> {
    if !path.exists() {
        return Ok(None);
    }
    if !path.is_file() {
        let _ = fs::remove_file(temp_path);
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "runtime session archive target path is not a file",
        )
        .into());
    }
    let backup_path = temporary_archive_path(path, "bak");
    if let Err(error) = fs::rename(path, &backup_path) {
        let _ = fs::remove_file(temp_path);
        return Err(error.into());
    }
    Ok(Some(backup_path))
}

fn restore_existing_target_backup(path: &Path, backup_path: Option<&Path>) {
    if let Some(backup_path) = backup_path {
        if path.exists() {
            let _ = fs::remove_file(path);
        }
        let _ = fs::rename(backup_path, path);
    }
}
