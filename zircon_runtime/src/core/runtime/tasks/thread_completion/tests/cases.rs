use std::cell::RefCell;
use std::sync::mpsc;
use std::time::Duration;

use super::thread_is_join_ready;

struct ThreadExitLatch {
    entered: mpsc::SyncSender<()>,
    release: mpsc::Receiver<()>,
}

impl Drop for ThreadExitLatch {
    fn drop(&mut self) {
        let _ = self.entered.send(());
        let _ = self.release.recv_timeout(Duration::from_secs(2));
    }
}

thread_local! {
    static EXIT_LATCH: RefCell<Option<ThreadExitLatch>> = const { RefCell::new(None) };
}

#[test]
fn native_join_readiness_rejects_a_finished_body_with_pending_tls_destruction() {
    let (entered, entered_receiver) = mpsc::sync_channel(1);
    let (release, release_receiver) = mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        EXIT_LATCH.with(|slot| {
            *slot.borrow_mut() = Some(ThreadExitLatch {
                entered,
                release: release_receiver,
            });
        });
    });
    let tls_entered = entered_receiver.recv_timeout(Duration::from_secs(1));
    let body_finished = worker.is_finished();
    let join_ready = thread_is_join_ready(&worker);
    let _ = release.send(());
    worker.join().unwrap();

    assert!(tls_entered.is_ok());
    assert!(body_finished);
    assert!(!join_ready);
}
