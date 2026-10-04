//! Serializes one project's generation preparation and file commit across runtime images.
//!
//! The in-image FIFO gate keeps same-image managers fair, while a persistent OS file lock covers
//! separately loaded Runtime images and other processes for the full preparation-to-commit span.

use std::collections::{BTreeSet, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex, OnceLock};

use super::{ProjectPaths, ResolvedProjectPathIdentity};

const GENERATION_LOCK_FILE_NAME: &str = ".project-generation-preparation.zrlock";

static AUTHORITY: OnceLock<ProjectGenerationAuthority> = OnceLock::new();

#[derive(Default)]
struct ProjectGenerationAuthority {
    state: Mutex<ProjectGenerationState>,
    changed: Condvar,
}

#[derive(Default)]
struct ProjectGenerationState {
    active: BTreeSet<ResolvedProjectPathIdentity>,
    waiters: VecDeque<(u64, ResolvedProjectPathIdentity)>,
    next_ticket: u64,
}

pub(crate) struct ProjectGenerationGuard {
    identity: ResolvedProjectPathIdentity,
    lock_file: Option<File>,
}

pub(crate) fn lock_project_generation(root: &Path) -> io::Result<ProjectGenerationGuard> {
    let identity = ProjectPaths::resolve_identity(root)?;
    let authority = AUTHORITY.get_or_init(ProjectGenerationAuthority::default);
    let mut state = authority
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let ticket = state.next_ticket;
    state.next_ticket = state.next_ticket.checked_add(1).ok_or_else(|| {
        io::Error::other("project generation authority exhausted its waiter ticket space")
    })?;
    state.waiters.push_back((ticket, identity.clone()));

    loop {
        let position = state
            .waiters
            .iter()
            .position(|(waiting_ticket, _)| *waiting_ticket == ticket)
            .expect("queued project generation waiter must remain registered");
        if !state.active.contains(&identity)
            && !state
                .waiters
                .iter()
                .take(position)
                .any(|(_, waiting_identity)| waiting_identity == &identity)
        {
            state.waiters.remove(position);
            state.active.insert(identity.clone());
            drop(state);

            let mut guard = ProjectGenerationGuard {
                identity,
                lock_file: None,
            };
            let lock_file = open_project_generation_lock(guard.identity.operation_path())?;
            File::lock(&lock_file)?;
            guard.lock_file = Some(lock_file);
            return Ok(guard);
        }
        state = authority
            .changed
            .wait(state)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }
}

impl Drop for ProjectGenerationGuard {
    fn drop(&mut self) {
        if let Some(lock_file) = self.lock_file.take() {
            let _ = File::unlock(&lock_file);
            drop(lock_file);
        }
        let authority = AUTHORITY
            .get()
            .expect("a generation guard cannot outlive its authority");
        let mut state = authority
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let removed = state.active.remove(&self.identity);
        debug_assert!(removed, "generation guard must own its project");
        drop(state);
        authority.changed.notify_all();
    }
}

fn open_project_generation_lock(root: &Path) -> io::Result<File> {
    let lock_path = project_generation_lock_path(root);
    let lock_parent = lock_path
        .parent()
        .expect("project generation lock path must have a parent directory");
    fs::create_dir_all(lock_parent)?;
    let (file, created) = open_generation_lock_file(&lock_path)?;
    if created {
        file.sync_all()?;
        crate::core::resource::io::sync_parent_directory(&lock_path)?;
    }
    Ok(file)
}

fn project_generation_lock_path(root: &Path) -> PathBuf {
    root.join(".zircon").join(GENERATION_LOCK_FILE_NAME)
}

fn open_generation_lock_file(path: &Path) -> io::Result<(File, bool)> {
    match OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(file) => Ok((file, true)),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let metadata = fs::symlink_metadata(path)?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "project generation lock must be a regular non-link file",
                ));
            }
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map(|file| (file, false))
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
#[path = "tests/generation_authority.rs"]
mod tests;
