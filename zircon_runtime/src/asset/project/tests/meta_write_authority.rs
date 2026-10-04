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
        .expect("the earlier multi-path writer must acquire after its active conflict releases");
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
