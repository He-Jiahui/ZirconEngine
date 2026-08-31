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
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{mpsc, Arc, Barrier};
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use super::{
        acquire_meta_paths_with_wait_hook, lock_meta_document_path, AssetMetaWriteGuard,
        AssetMetaWriteGuards,
    };

    static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn one_resolved_meta_identity_admits_only_one_writer() {
        let root = unique_test_root("same-identity");
        let first = root.join("assets/panel.zui.zmeta");
        let alias = root.join("assets/../assets/panel.zui.zmeta");
        let active = lock_meta_document_path(&first).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let worker_barrier = Arc::clone(&barrier);
        let (acquired_send, acquired_receive) = mpsc::channel();
        let worker = thread::spawn(move || {
            worker_barrier.wait();
            let waiting = lock_meta_document_path(&alias).unwrap();
            acquired_send.send(()).unwrap();
            drop(waiting);
        });

        barrier.wait();
        assert!(matches!(
            acquired_receive.recv_timeout(Duration::from_millis(100)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));
        drop(active);
        acquired_receive
            .recv_timeout(Duration::from_secs(5))
            .expect("alias writer must acquire after the active identity is released");
        worker.join().unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unrelated_meta_identities_do_not_share_a_false_lock_stripe() {
        let root = unique_test_root("independent-identities");
        let active = lock_meta_document_path(&root.join("assets/first.zmeta")).unwrap();
        let second = root.join("assets/second.zmeta");
        let barrier = Arc::new(Barrier::new(2));
        let worker_barrier = Arc::clone(&barrier);
        let (acquired_send, acquired_receive) = mpsc::channel();
        let worker = thread::spawn(move || {
            worker_barrier.wait();
            let independent = lock_meta_document_path(&second).unwrap();
            acquired_send.send(()).unwrap();
            drop(independent);
        });

        barrier.wait();
        let acquired = acquired_receive.recv_timeout(Duration::from_secs(5));
        drop(active);
        worker.join().unwrap();
        acquired.expect("unrelated meta identities must acquire independently");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn earlier_conflicting_multi_path_waiter_cannot_be_barged_by_a_later_writer() {
        let root = unique_test_root("fair-conflicting-waiters");
        let first = root.join("assets/first.zmeta");
        let second = root.join("assets/second.zmeta");
        let active = lock_meta_document_path(&first).unwrap();

        let (waiting_send, waiting_receive) = mpsc::channel();
        let (earlier_acquired_send, earlier_acquired_receive) = mpsc::channel();
        let (release_earlier_send, release_earlier_receive) = mpsc::channel();
        let earlier_first = first.clone();
        let earlier_second = second.clone();
        let earlier = thread::spawn(move || {
            let identities = acquire_meta_paths_with_wait_hook(
                [earlier_first.as_path(), earlier_second.as_path()],
                || waiting_send.send(()).unwrap(),
            )
            .unwrap();
            let guard = AssetMetaWriteGuards { identities };
            earlier_acquired_send.send(()).unwrap();
            release_earlier_receive.recv().unwrap();
            drop(guard);
        });
        waiting_receive
            .recv_timeout(Duration::from_secs(5))
            .expect("the earlier multi-path writer must be queued before the later writer starts");

        let (later_waiting_send, later_waiting_receive) = mpsc::channel();
        let (later_acquired_send, later_acquired_receive) = mpsc::channel();
        let later = thread::spawn(move || {
            let identities = acquire_meta_paths_with_wait_hook([second.as_path()], || {
                later_waiting_send.send(()).unwrap()
            })
            .unwrap();
            let guard = AssetMetaWriteGuard { identities };
            later_acquired_send.send(()).unwrap();
            drop(guard);
        });
        later_waiting_receive
            .recv_timeout(Duration::from_secs(5))
            .expect("the later conflicting writer must be queued before the active guard releases");
        assert!(matches!(
            later_acquired_receive.recv_timeout(Duration::from_millis(100)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));

        drop(active);
        earlier_acquired_receive
            .recv_timeout(Duration::from_secs(5))
            .expect(
                "the earlier multi-path writer must acquire after its active conflict releases",
            );
        assert!(matches!(
            later_acquired_receive.recv_timeout(Duration::from_millis(100)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));

        release_earlier_send.send(()).unwrap();
        later_acquired_receive
            .recv_timeout(Duration::from_secs(5))
            .expect("the later writer must acquire after the earlier conflicting waiter releases");
        earlier.join().unwrap();
        later.join().unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn later_disjoint_writer_can_pass_an_earlier_blocked_waiter() {
        let root = unique_test_root("disjoint-waiter-progress");
        let first = root.join("assets/first.zmeta");
        let second = root.join("assets/second.zmeta");
        let active = lock_meta_document_path(&first).unwrap();

        let (earlier_waiting_send, earlier_waiting_receive) = mpsc::channel();
        let (earlier_acquired_send, earlier_acquired_receive) = mpsc::channel();
        let earlier = thread::spawn(move || {
            let identities = acquire_meta_paths_with_wait_hook([first.as_path()], || {
                earlier_waiting_send.send(()).unwrap()
            })
            .unwrap();
            let guard = AssetMetaWriteGuard { identities };
            earlier_acquired_send.send(()).unwrap();
            drop(guard);
        });
        earlier_waiting_receive
            .recv_timeout(Duration::from_secs(5))
            .expect("the earlier conflicting writer must be queued first");

        let (later_acquired_send, later_acquired_receive) = mpsc::channel();
        let later = thread::spawn(move || {
            let guard = lock_meta_document_path(&second).unwrap();
            later_acquired_send.send(()).unwrap();
            drop(guard);
        });
        later_acquired_receive
            .recv_timeout(Duration::from_secs(5))
            .expect("a disjoint writer must not wait behind an earlier blocked request");

        drop(active);
        earlier_acquired_receive
            .recv_timeout(Duration::from_secs(5))
            .expect("the earlier writer must acquire after its active conflict releases");
        earlier.join().unwrap();
        later.join().unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    fn unique_test_root(label: &str) -> PathBuf {
        let root = test_output_root().join(format!(
            "zircon-meta-write-{label}-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time after Unix epoch")
                .as_nanos(),
            NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("assets")).unwrap();
        root
    }

    fn test_output_root() -> PathBuf {
        std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
            .or_else(|| std::env::var_os("CARGO_TARGET_DIR"))
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::current_dir()
                    .expect("resolve current workspace for meta-write test output")
                    .join("target")
            })
            .join("zircon-test-output")
    }
}
