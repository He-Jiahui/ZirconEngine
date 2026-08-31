use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, Weak};

use crate::asset::project::{ProjectPaths, ResolvedProjectPath, ResolvedProjectPathIdentity};
use crate::core::resource::io::stage_atomic_write;

use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveArtifact, RuntimeSessionArchiveError, archive_save,
};

static ARCHIVE_PATH_WRITE_AUTHORITY: OnceLock<ArchivePathWriteAuthority> = OnceLock::new();

#[derive(Default)]
struct ArchivePathWriteAuthority {
    paths: Mutex<BTreeMap<ResolvedProjectPathIdentity, Weak<ArchivePathWriteState>>>,
}

struct ArchivePathWriteState {
    identity: ResolvedProjectPathIdentity,
    revision: Mutex<CommittedPathRevision>,
}

#[derive(Clone, Copy, Default)]
struct CommittedPathRevision {
    commit: u64,
    next_write_generation: u64,
    admitted_write_generation: u64,
}

pub(super) struct ArchivePathWriteReservation {
    state: Arc<ArchivePathWriteState>,
    write_generation: u64,
}

pub(super) struct ArchivePathWriteTicket {
    state: Arc<ArchivePathWriteState>,
    expected_commit: u64,
    write_generation: u64,
}

impl ArchivePathWriteReservation {
    pub(super) const fn write_generation(&self) -> u64 {
        self.write_generation
    }
}

impl ArchivePathWriteAuthority {
    fn state_for(&self, identity: ResolvedProjectPathIdentity) -> Arc<ArchivePathWriteState> {
        let mut paths = self.lock_paths();
        if let Some(state) = paths.get(&identity).and_then(Weak::upgrade) {
            return state;
        }

        let state = Arc::new(ArchivePathWriteState {
            identity: identity.clone(),
            revision: Mutex::new(CommittedPathRevision::default()),
        });
        paths.insert(identity, Arc::downgrade(&state));
        state
    }

    fn lock_paths(
        &self,
    ) -> MutexGuard<'_, BTreeMap<ResolvedProjectPathIdentity, Weak<ArchivePathWriteState>>> {
        self.paths
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn retire(&self, state: &ArchivePathWriteState) {
        let mut paths = self.lock_paths();
        if paths
            .get(&state.identity)
            .is_some_and(|registered| std::ptr::eq(registered.as_ptr(), std::ptr::from_ref(state)))
        {
            paths.remove(&state.identity);
        }
    }
}

impl ArchivePathWriteState {
    fn lock_revision(&self) -> MutexGuard<'_, CommittedPathRevision> {
        self.revision
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Drop for ArchivePathWriteState {
    fn drop(&mut self) {
        archive_path_write_authority().retire(self);
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
    let path = ProjectPaths::resolve_path(path)?;
    let identity = ResolvedProjectPathIdentity::from(path.clone());
    let reservation = reserve_archive_path_write(artifact, identity)?;
    let ticket = admit_archive_path_write(artifact, reservation)?;
    save_artifact_to_prepared_path_atomically(artifact, &path, &ticket)
}

pub(super) fn save_artifact_to_prepared_path_atomically(
    artifact: &RuntimeSessionArchiveArtifact,
    path: &ResolvedProjectPath,
    ticket: &ArchivePathWriteTicket,
) -> Result<(), RuntimeSessionArchiveError> {
    let operation_path = path.operation_path();
    archive_save::preview_artifact_save_to_path(artifact, operation_path)?;
    let staged = stage_atomic_write(operation_path, artifact.serialized_bytes())?;

    let mut committed = ticket.state.lock_revision();
    let current = *committed;
    if current.admitted_write_generation != ticket.write_generation {
        return Err(RuntimeSessionArchiveError::StalePathWrite {
            write_generation: ticket.write_generation,
            current_generation: current.admitted_write_generation,
        });
    }
    if current.commit != ticket.expected_commit {
        return Err(RuntimeSessionArchiveError::StalePathCommit {
            expected_commit: ticket.expected_commit,
            committed_commit: current.commit,
        });
    }
    reject_stale_lineage_revision(artifact)?;
    let _publication = artifact.lock_publication();
    reject_stale_lineage_revision(artifact)?;

    let commit_result = staged.commit();
    let published = commit_result.is_ok()
        || file_matches_bytes(operation_path, artifact.serialized_bytes()).unwrap_or(false);
    if published {
        committed.commit = current.commit.saturating_add(1);
        artifact.record_published_revision();
    }
    commit_result.map_err(Into::into)
}

fn file_matches_bytes(path: &Path, expected: &[u8]) -> io::Result<bool> {
    let mut file = fs::File::open(path)?;
    let expected_len = u64::try_from(expected.len()).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "runtime session archive byte length exceeds the filesystem range",
        )
    })?;
    if file.metadata()?.len() != expected_len {
        return Ok(false);
    }

    let mut buffer = [0u8; 64 * 1024];
    let mut offset = 0usize;
    while offset < expected.len() {
        let remaining = expected.len() - offset;
        let chunk_len = remaining.min(buffer.len());
        let read = file.read(&mut buffer[..chunk_len])?;
        if read == 0 || buffer[..read] != expected[offset..offset + read] {
            return Ok(false);
        }
        offset += read;
    }
    Ok(true)
}

pub(super) fn reserve_archive_path_write(
    artifact: &RuntimeSessionArchiveArtifact,
    identity: ResolvedProjectPathIdentity,
) -> Result<ArchivePathWriteReservation, RuntimeSessionArchiveError> {
    reject_stale_lineage_revision(artifact)?;
    let state = archive_path_write_authority().state_for(identity);
    let write_generation = {
        let mut committed = state.lock_revision();
        committed.next_write_generation = committed.next_write_generation.saturating_add(1);
        committed.next_write_generation
    };
    Ok(ArchivePathWriteReservation {
        state,
        write_generation,
    })
}

pub(super) fn admit_archive_path_write(
    artifact: &RuntimeSessionArchiveArtifact,
    reservation: ArchivePathWriteReservation,
) -> Result<ArchivePathWriteTicket, RuntimeSessionArchiveError> {
    reject_stale_lineage_revision(artifact)?;
    let expected_commit = {
        let mut committed = reservation.state.lock_revision();
        if reservation.write_generation <= committed.admitted_write_generation {
            return Err(RuntimeSessionArchiveError::StalePathWrite {
                write_generation: reservation.write_generation,
                current_generation: committed.admitted_write_generation,
            });
        }
        committed.admitted_write_generation = reservation.write_generation;
        committed.commit
    };
    Ok(ArchivePathWriteTicket {
        state: reservation.state,
        expected_commit,
        write_generation: reservation.write_generation,
    })
}

fn reject_stale_lineage_revision(
    artifact: &RuntimeSessionArchiveArtifact,
) -> Result<(), RuntimeSessionArchiveError> {
    let committed_revision = artifact.latest_published_revision();
    if artifact.revision() < committed_revision {
        return Err(RuntimeSessionArchiveError::StaleArtifactRevision {
            artifact_revision: artifact.revision(),
            committed_revision,
        });
    }
    Ok(())
}

fn archive_path_write_authority() -> &'static ArchivePathWriteAuthority {
    ARCHIVE_PATH_WRITE_AUTHORITY.get_or_init(ArchivePathWriteAuthority::default)
}

#[cfg(test)]
pub(super) fn archive_path_write_authority_contains(
    identity: &ResolvedProjectPathIdentity,
) -> bool {
    archive_path_write_authority()
        .lock_paths()
        .contains_key(identity)
}
