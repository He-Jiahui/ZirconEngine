use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use super::{OwnerDispatchError, OwnerShutdownReceipt, SessionOwner};

struct AffineState {
    _not_send: Rc<()>,
    owner: ThreadId,
    drops: Arc<Mutex<Vec<ThreadId>>>,
}

impl Drop for AffineState {
    fn drop(&mut self) {
        assert_eq!(self.owner, thread::current().id());
        self.drops.lock().unwrap().push(thread::current().id());
    }
}

#[test]
fn astra_life_a4_managed_owner_keeps_non_send_state_and_pending_cleanup_on_one_thread() {
    let drops = Arc::new(Mutex::new(Vec::new()));
    let drops_for_factory = Arc::clone(&drops);
    let library_guard = Arc::new(());
    let library_guard_weak = Arc::downgrade(&library_guard);
    let owner = SessionOwner::create(
        move || AffineState {
            _not_send: Rc::new(()),
            owner: thread::current().id(),
            drops: drops_for_factory,
        },
        library_guard,
    )
    .unwrap();
    let owner_thread = owner
        .dispatch(Instant::now() + Duration::from_secs(1), |state| {
            assert_eq!(state.owner, thread::current().id());
            state.owner
        })
        .unwrap()
        .recv_timeout(Duration::from_secs(1))
        .unwrap();
    assert_ne!(owner_thread, thread::current().id());

    let invocations = Arc::new(AtomicUsize::new(0));
    let cleanup_invocations = Arc::clone(&invocations);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let first_deadline = Instant::now() + Duration::from_millis(25);
    assert_eq!(
        owner.shutdown_until(first_deadline, move |state| {
            assert_eq!(state.owner, thread::current().id());
            cleanup_invocations.fetch_add(1, Ordering::SeqCst);
            release_rx.recv().unwrap();
            true
        }),
        OwnerShutdownReceipt::Pending
    );
    assert!(library_guard_weak.upgrade().is_some());
    assert!(drops.lock().unwrap().is_empty());
    assert!(!owner.task_is_terminal());
    assert_eq!(
        owner.shutdown_until(Instant::now() + Duration::from_millis(25), |_| {
            panic!("pending cleanup must never be reinvoked")
        }),
        OwnerShutdownReceipt::Pending
    );

    release_tx.send(()).unwrap();
    assert_eq!(
        owner.shutdown_until(Instant::now() + Duration::from_secs(1), |_| {
            panic!("pending cleanup must never be reinvoked")
        }),
        OwnerShutdownReceipt::Joined
    );
    assert_eq!(invocations.load(Ordering::SeqCst), 1);
    assert_eq!(*drops.lock().unwrap(), vec![owner_thread]);
    assert!(library_guard_weak.upgrade().is_some());
    assert!(owner.task_is_terminal());
    drop(owner);
    assert!(library_guard_weak.upgrade().is_none());
}

#[test]
fn astra_life_a4_owner_cleanup_failure_retains_state_for_explicit_retry() {
    let calls = Arc::new(AtomicUsize::new(0));
    let first_calls = Arc::clone(&calls);
    let owner = SessionOwner::create(|| Rc::new(()), Arc::new(())).unwrap();
    assert_eq!(
        owner.shutdown_until(Instant::now() + Duration::from_secs(1), move |_| {
            first_calls.fetch_add(1, Ordering::SeqCst);
            false
        }),
        OwnerShutdownReceipt::Retryable
    );
    assert!(owner.dispatch(Instant::now(), |_| ()).is_err());
    let retry_calls = Arc::clone(&calls);
    assert_eq!(
        owner.shutdown_until(Instant::now() + Duration::from_secs(1), move |_| {
            retry_calls.fetch_add(1, Ordering::SeqCst);
            true
        }),
        OwnerShutdownReceipt::Joined
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn astra_life_a4_owner_cleanup_panic_requires_explicit_recovery_and_retains_code_owner() {
    let code_owner = Arc::new(());
    let retained = Arc::downgrade(&code_owner);
    let owner = SessionOwner::create(|| Rc::new(()), code_owner).unwrap();
    assert_eq!(
        owner.shutdown_until(Instant::now() + Duration::from_secs(1), |_| {
            panic!("cleanup failed")
        }),
        OwnerShutdownReceipt::Poisoned
    );
    assert_eq!(
        owner.shutdown_until(Instant::now(), |_| panic!("normal retry cannot run")),
        OwnerShutdownReceipt::Poisoned
    );
    assert!(retained.upgrade().is_some());
    assert!(!owner.task_is_terminal());
    assert_eq!(
        owner
            .recover_poisoned_until(Instant::now() + Duration::from_secs(1), |_| true)
            .unwrap(),
        OwnerShutdownReceipt::Joined
    );
    drop(owner);
    assert!(retained.upgrade().is_none());
}

#[test]
fn astra_life_a4_owner_factory_panic_has_joinable_terminal_receipt() {
    let owner = SessionOwner::<Rc<()>>::create(|| panic!("factory failed"), Arc::new(())).unwrap();
    let deadline = Instant::now() + Duration::from_secs(1);
    while !owner.task_is_terminal() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(owner.task_is_terminal());
    assert_eq!(
        owner.shutdown_until(deadline, |_| true),
        OwnerShutdownReceipt::Panicked
    );
    drop(owner);
}

#[test]
fn astra_life_a4_scoped_dispatch_completes_before_borrowed_output_is_released() {
    let owner = SessionOwner::create(|| 41_u32, Arc::new(())).unwrap();
    let mut output = Vec::new();
    owner
        .dispatch_scoped(|state| output.push(*state + 1))
        .unwrap();
    assert_eq!(output, vec![42]);
    assert_eq!(
        owner.shutdown_until(Instant::now() + Duration::from_secs(1), |_| true),
        OwnerShutdownReceipt::Joined
    );
}

#[test]
fn astra_life_a4_scoped_dispatch_rejects_same_owner_reentry_and_remains_usable() {
    let owner = SessionOwner::create(|| 0_u32, Arc::new(())).unwrap();
    assert_eq!(
        owner.dispatch_scoped(|_| owner.dispatch_scoped(|_| ())),
        Ok(Err(OwnerDispatchError::Reentrant))
    );
    assert_eq!(owner.dispatch_scoped(|state| *state += 1), Ok(()));
    assert_eq!(
        owner.shutdown_until(Instant::now() + Duration::from_secs(1), |state| *state == 1),
        OwnerShutdownReceipt::Joined
    );
}
