//! sidecar 写入使用进程内路径身份队列协调；导入、预览、迁移与重定位共用此锁，批量路径按同一顺序取得以避免相互覆盖。

use std::collections::{BTreeSet, VecDeque};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex, MutexGuard, OnceLock};

use super::{ProjectPaths, ResolvedProjectPathIdentity};

static META_WRITE_AUTHORITY: OnceLock<MetaWriteAuthority> = OnceLock::new();

#[derive(Debug, Default)]
struct MetaWriteAuthority {
    state: Mutex<MetaWriteState>,
    state_changed: Condvar,
}

#[derive(Debug, Default)]
struct MetaWriteState {
    active_paths: BTreeSet<ResolvedProjectPathIdentity>,
    waiters: VecDeque<MetaWriteWaiter>,
    next_ticket: u64,
}

#[derive(Debug)]
struct MetaWriteWaiter {
    ticket: u64,
    identities: Vec<ResolvedProjectPathIdentity>,
}

pub(crate) struct AssetMetaWriteGuard {
    identities: Vec<ResolvedProjectPathIdentity>,
}

pub(crate) struct AssetMetaWriteGuards {
    identities: Vec<ResolvedProjectPathIdentity>,
}

pub(crate) fn lock_meta_document_path(path: &Path) -> io::Result<AssetMetaWriteGuard> {
    Ok(AssetMetaWriteGuard {
        identities: acquire_meta_paths(std::iter::once(path))?,
    })
}

/// 为同一事务中的多个 sidecar 取得统一的写入权；持有 guard 期间再比较磁盘前置条件并提交。
pub(crate) fn lock_meta_document_paths(paths: &[PathBuf]) -> io::Result<AssetMetaWriteGuards> {
    Ok(AssetMetaWriteGuards {
        identities: acquire_meta_paths(paths.iter().map(PathBuf::as_path))?,
    })
}

fn acquire_meta_paths<'a>(
    paths: impl IntoIterator<Item = &'a Path>,
) -> io::Result<Vec<ResolvedProjectPathIdentity>> {
    acquire_meta_paths_with_wait_hook(paths, || {})
}

fn acquire_meta_paths_with_wait_hook<'a>(
    paths: impl IntoIterator<Item = &'a Path>,
    on_wait: impl FnOnce(),
) -> io::Result<Vec<ResolvedProjectPathIdentity>> {
    let identities = paths
        .into_iter()
        .map(ProjectPaths::resolve_identity)
        .collect::<io::Result<BTreeSet<_>>>()?
        .into_iter()
        .collect::<Vec<_>>();
    let authority = authority();
    let mut state = authority.lock_state();
    let ticket = state.next_ticket;
    state.next_ticket = state.next_ticket.checked_add(1).ok_or_else(|| {
        io::Error::other("asset meta write authority exhausted its waiter ticket space")
    })?;
    state
        .waiters
        .push_back(MetaWriteWaiter { ticket, identities });
    let mut on_wait = Some(on_wait);

    loop {
        let position = state
            .waiters
            .iter()
            .position(|waiter| waiter.ticket == ticket)
            .expect("queued asset meta write waiter must remain registered");
        let waiter = &state.waiters[position];
        let active_conflict = waiter
            .identities
            .iter()
            .any(|identity| state.active_paths.contains(identity));
        if !active_conflict
            && !earlier_waiter_conflicts(&state.waiters, position, &waiter.identities)
        {
            let waiter = state
                .waiters
                .remove(position)
                .expect("eligible asset meta write waiter must remain registered");
            state.active_paths.extend(waiter.identities.iter().cloned());
            return Ok(waiter.identities);
        }

        if let Some(on_wait) = on_wait.take() {
            drop(state);
            on_wait();
            state = authority.lock_state();
            continue;
        }
        state = authority
            .state_changed
            .wait(state)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }
}

fn earlier_waiter_conflicts(
    waiters: &VecDeque<MetaWriteWaiter>,
    position: usize,
    identities: &[ResolvedProjectPathIdentity],
) -> bool {
    waiters
        .iter()
        .take(position)
        .any(|waiter| sorted_identities_conflict(&waiter.identities, identities))
}

fn sorted_identities_conflict(
    left: &[ResolvedProjectPathIdentity],
    right: &[ResolvedProjectPathIdentity],
) -> bool {
    let (mut left_index, mut right_index) = (0, 0);
    while left_index < left.len() && right_index < right.len() {
        match left[left_index].cmp(&right[right_index]) {
            std::cmp::Ordering::Less => left_index += 1,
            std::cmp::Ordering::Greater => right_index += 1,
            std::cmp::Ordering::Equal => return true,
        }
    }
    false
}

impl MetaWriteAuthority {
    fn lock_state(&self) -> MutexGuard<'_, MetaWriteState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn release(&self, identities: &[ResolvedProjectPathIdentity]) {
        let mut state = self.lock_state();
        for identity in identities {
            let removed = state.active_paths.remove(identity);
            debug_assert!(removed, "asset meta write guard must own its path");
        }
        drop(state);
        self.state_changed.notify_all();
    }
}

impl Drop for AssetMetaWriteGuard {
    fn drop(&mut self) {
        authority().release(&self.identities);
    }
}

impl Drop for AssetMetaWriteGuards {
    fn drop(&mut self) {
        authority().release(&self.identities);
    }
}

fn authority() -> &'static MetaWriteAuthority {
    META_WRITE_AUTHORITY.get_or_init(MetaWriteAuthority::default)
}

#[cfg(test)]
#[path = "tests/meta_write_authority.rs"]
mod tests;
