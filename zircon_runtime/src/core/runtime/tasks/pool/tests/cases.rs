use std::sync::mpsc;
use std::time::Duration;

use super::{TaskPool, TaskPoolDescriptor};

#[test]
fn owner_shutdown_deadline_is_retryable_while_a_submission_is_active() {
    use std::time::Instant;
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let (release, wait) = mpsc::sync_channel(1);
    pool.spawn(move || {
        wait.recv_timeout(Duration::from_secs(2)).unwrap();
    });
    assert!(!pool.shutdown_until(Instant::now()));
    assert!(pool.try_acquire_submission().is_none());
    release.send(()).unwrap();
    assert!(pool.shutdown_until(Instant::now() + Duration::from_secs(2)));
    assert!(pool.shutdown_until(Instant::now()));
}

#[test]
fn acquired_submission_survives_external_admission_close() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let submission = pool
        .try_acquire_submission()
        .expect("open pool should issue a submission authority");
    pool.close_admission();
    let (sender, receiver) = mpsc::sync_channel(1);

    submission.spawn(move || sender.send(()).expect("submission result"));

    receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("work accepted before close must still reach the worker");
}

#[cfg(windows)]
#[test]
fn task_pool_retains_unjoined_worker_while_its_tls_destructor_is_blocked() {
    use std::cell::RefCell;
    use std::time::Instant;

    struct ExitLatch(mpsc::SyncSender<()>, mpsc::Receiver<()>);
    impl Drop for ExitLatch {
        fn drop(&mut self) {
            let _ = self.0.send(());
            let _ = self.1.recv_timeout(Duration::from_secs(2));
        }
    }
    thread_local! {
        static EXIT_LATCH: RefCell<Option<ExitLatch>> = const { RefCell::new(None) };
    }

    let pool = TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(1));
    let (installed, installed_receiver) = mpsc::sync_channel(1);
    let (entered, entered_receiver) = mpsc::sync_channel(1);
    let (release, release_receiver) = mpsc::sync_channel(1);
    pool.spawn(move || {
        EXIT_LATCH.with(|slot| {
            *slot.borrow_mut() = Some(ExitLatch(entered, release_receiver));
        });
        installed.send(()).unwrap();
    });
    installed_receiver
        .recv_timeout(Duration::from_secs(1))
        .unwrap();
    let started_at = Instant::now();
    let pending = pool.close_and_join(Duration::from_millis(25));
    let elapsed = started_at.elapsed();
    let tls_entered = entered_receiver.recv_timeout(Duration::from_secs(1));
    let retry_pending = pool.close_and_join(Duration::ZERO);
    let _ = release.send(());
    let joined = pool.close_and_join(Duration::from_secs(1));

    assert!(tls_entered.is_ok());
    assert!(elapsed < Duration::from_secs(1));
    assert_eq!(pending.expected_worker_count, 1);
    assert_eq!(pending.joined_worker_count, 0);
    assert_eq!(retry_pending.exited_worker_count, 1);
    assert_eq!(retry_pending.joined_worker_count, 0);
    assert_eq!(joined.joined_worker_count, 1);
    assert_eq!(pool.close_and_join(Duration::ZERO).joined_worker_count, 1);
}
