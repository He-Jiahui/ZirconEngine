//! 通用 owner 的线程归属、清理重试与借用回调契约测试。
//!
//! 使用真实单 worker 池和屏障核对状态销毁、代码 owner 保留及回执阶段。

use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use super::{OwnerDispatchError, OwnerShutdownReceipt, SessionOwner};

/// Rc 强制此状态为非 Send；构造、动作和 Drop 都须落在工厂记录的同一线程。
struct AffineState {
    _not_send: Rc<()>,
    owner: ThreadId,
    drops: Arc<Mutex<Vec<ThreadId>>>,
}

impl Drop for AffineState {
    fn drop(&mut self) {
        // 记录真正的析构线程，不以清理回调进入或 terminal 标志替代状态 Drop。
        assert_eq!(self.owner, thread::current().id());
        self.drops.lock().unwrap().push(thread::current().id());
    }
}

#[test]
/// 两次短等待共享一个在途清理，释放屏障后只执行一次并在同一线程销毁状态。
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

    // 清理回调被真实屏障阻塞；Pending 必须保留状态与代码 owner，而不是重新调用清理。
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

    // 释放原回调后重取同一回执；控制器 Drop 之前代码所有者仍然存活。
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
/// 清理返回 false 后仍保留状态，拒绝普通动作，并允许一个新的显式清理尝试。
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
/// 清理 panic 只形成 Poisoned；普通重试不重跑它，显式恢复成功后才释放代码 owner。
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
/// 工厂 panic 的 terminal 标志仍须经过真实池 join，Panicked 回执允许随后 Drop。
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
/// 动作借用调用栈上的 Vec，完成返回后即可安全读取该输出。
fn astra_life_a4_scoped_dispatch_completes_before_borrowed_output_is_released() {
    let owner = SessionOwner::create(|| 41_u32, Arc::new(())).unwrap();
    let mut output = Vec::new();
    owner
        .dispatch_scoped(Instant::now() + Duration::from_secs(1), |state| {
            output.push(*state + 1)
        })
        .unwrap();
    assert_eq!(output, vec![42]);
    assert_eq!(
        owner.shutdown_until(Instant::now() + Duration::from_secs(1), |_| true),
        OwnerShutdownReceipt::Joined
    );
}

#[test]
/// 同一 owner 的同步重入返回 Reentrant；外层完成后正常动作和关闭仍然可用。
fn astra_life_a4_scoped_dispatch_rejects_same_owner_reentry_and_remains_usable() {
    let owner = SessionOwner::create(|| 0_u32, Arc::new(())).unwrap();
    assert_eq!(
        owner.dispatch_scoped(Instant::now() + Duration::from_secs(1), |_| {
            owner.dispatch_scoped(Instant::now() + Duration::from_secs(1), |_| ())
        }),
        Ok(Err(OwnerDispatchError::Reentrant))
    );
    assert_eq!(
        owner.dispatch_scoped(Instant::now() + Duration::from_secs(1), |state| *state += 1),
        Ok(())
    );
    assert_eq!(
        owner.shutdown_until(Instant::now() + Duration::from_secs(1), |state| *state == 1),
        OwnerShutdownReceipt::Joined
    );
}

#[test]
fn scoped_admission_timeout_releases_shutdown_lock_and_drops_unadmitted_borrow() {
    let owner = SessionOwner::create(|| 0_u32, Arc::new(())).unwrap();
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    owner
        .dispatch(Instant::now() + Duration::from_secs(1), move |_| {
            entered_tx.send(()).unwrap();
            release_rx.recv().unwrap();
        })
        .unwrap();
    entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    for _ in 0..super::COMMAND_CAPACITY {
        owner.dispatch(Instant::now(), |_| ()).unwrap();
    }
    let mut output = Vec::new();
    let result = owner.dispatch_scoped(
        Instant::now() + Duration::from_millis(25),
        |_| output.push(42),
    );
    let shutdown_available = owner.shutdown.try_lock().is_ok();
    release_tx.send(()).unwrap();
    let shutdown = owner.shutdown_until(Instant::now() + Duration::from_secs(2), |_| true);
    assert_eq!(shutdown, OwnerShutdownReceipt::Joined);
    assert_eq!(result, Err(OwnerDispatchError::AdmissionIncomplete));
    assert!(shutdown_available);
    assert!(output.is_empty());
}

#[test]
fn admitted_scoped_action_waits_for_completion_beyond_admission_deadline() {
    let owner = SessionOwner::create(|| 41_u32, Arc::new(())).unwrap();
    let mut output = Vec::new();
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let (finished_tx, finished_rx) = mpsc::sync_channel(1);
    thread::scope(|scope| {
        let owner = &owner;
        let output = &mut output;
        scope.spawn(move || {
            // An empty queue can admit immediately even with an expired deadline.
            let result = owner.dispatch_scoped(Instant::now(), move |state| {
                entered_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                output.push(*state + 1);
            });
            finished_tx.send(result).unwrap();
        });
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let completed_early = finished_rx.try_recv().is_ok();
        release_tx.send(()).unwrap();
        assert!(!completed_early);
        assert_eq!(
            finished_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            Ok(())
        );
    });
    assert_eq!(output, vec![42]);
    assert_eq!(
        owner.shutdown_until(Instant::now() + Duration::from_secs(1), |_| true),
        OwnerShutdownReceipt::Joined
    );
}
