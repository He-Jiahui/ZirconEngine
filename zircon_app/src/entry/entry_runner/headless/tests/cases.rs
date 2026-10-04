use std::num::NonZeroU64;
use std::time::{Duration, Instant};

use super::controller::HeadlessWait;
use super::schedule::HeadlessSchedule;
use super::session::{drive, HeadlessSession};
use super::{args, HeadlessController, HeadlessHostError, HeadlessStopReason};
use crate::entry::product_shutdown::ProductFailureLedger;

#[cfg(windows)]
struct ThreadExitLatch {
    entered: std::sync::mpsc::SyncSender<()>,
    release: std::sync::mpsc::Receiver<()>,
}

#[cfg(windows)]
impl Drop for ThreadExitLatch {
    fn drop(&mut self) {
        let _ = self.entered.send(());
        let _ = self.release.recv_timeout(Duration::from_secs(2));
    }
}

#[cfg(windows)]
thread_local! {
    static THREAD_EXIT_LATCH: std::cell::RefCell<Option<ThreadExitLatch>> = const { std::cell::RefCell::new(None) };
}

#[derive(Default)]
struct FakeSession {
    ticks: u64,
    drains: u64,
    stops: u64,
    fail_tick: bool,
    fail_stop: bool,
    fail_composition: bool,
    fail_drain: bool,
    stop_failures: u64,
    cancel_after_tick: Option<HeadlessController>,
}

impl HeadlessSession for FakeSession {
    fn verify_composition(&self) -> Result<(), HeadlessHostError> {
        if self.fail_composition {
            Err(HeadlessHostError::Startup(
                "composition not ready".to_owned(),
            ))
        } else {
            Ok(())
        }
    }
    fn tick(&mut self) -> Result<(), HeadlessHostError> {
        if self.fail_tick {
            return Err(HeadlessHostError::Runtime(
                "primary tick failure".to_owned(),
            ));
        }
        self.ticks += 1;
        if let Some(controller) = &self.cancel_after_tick {
            controller.cancel();
        }
        Ok(())
    }
    fn drain_host(&mut self) -> Result<(), HeadlessHostError> {
        self.drains += 1;
        if self.fail_drain {
            return Err(HeadlessHostError::Runtime("host drain failed".to_owned()));
        }
        Ok(())
    }
    fn stop(&mut self) -> Result<(), HeadlessHostError> {
        self.stops += 1;
        if self.fail_stop || self.stops <= self.stop_failures {
            Err(HeadlessHostError::Runtime(
                "secondary stop failure".to_owned(),
            ))
        } else {
            Ok(())
        }
    }
}

#[test]
fn headless_host_runs_exact_tick_count_and_stops_once() {
    let mut session = FakeSession::default();
    let report = drive(
        &mut session,
        &HeadlessController::default(),
        NonZeroU64::new(3),
        Duration::from_nanos(1),
        &ProductFailureLedger::default(),
    )
    .unwrap();
    assert_eq!(report.completed_ticks, 3);
    assert!(report.ready);
    assert_eq!(report.reason, HeadlessStopReason::TickLimit);
    assert_eq!((session.ticks, session.drains, session.stops), (3, 3, 1));
}

#[test]
fn headless_host_cancel_before_first_tick_never_reports_ready() {
    let controller = HeadlessController::default();
    controller.cancel();
    let mut session = FakeSession::default();
    let report = drive(
        &mut session,
        &controller,
        None,
        Duration::from_secs(1),
        &ProductFailureLedger::default(),
    )
    .unwrap();
    assert!(!report.ready);
    assert_eq!(report.reason, HeadlessStopReason::Cancelled);
    assert_eq!((session.ticks, session.stops), (0, 1));
}

#[test]
fn headless_host_cancel_after_tick_does_not_wait_for_next_deadline() {
    let controller = HeadlessController::default();
    let mut session = FakeSession {
        cancel_after_tick: Some(controller.clone()),
        ..Default::default()
    };
    let report = drive(
        &mut session,
        &controller,
        None,
        Duration::from_secs(60),
        &ProductFailureLedger::default(),
    )
    .unwrap();
    assert_eq!(report.completed_ticks, 1);
    assert_eq!(report.reason, HeadlessStopReason::Cancelled);
}

#[test]
fn headless_host_preserves_primary_tick_failure_before_stop_failure() {
    let failures = ProductFailureLedger::default();
    let mut session = FakeSession {
        fail_tick: true,
        fail_stop: true,
        ..Default::default()
    };
    drive(
        &mut session,
        &HeadlessController::default(),
        None,
        Duration::from_nanos(1),
        &failures,
    )
    .unwrap_err();
    let report = failures.snapshot();
    assert!(report
        .primary()
        .unwrap()
        .message()
        .contains("primary tick failure"));
    assert!(report.secondary()[0]
        .message()
        .contains("secondary stop failure"));
    assert_eq!(session.stops, 1);
}

#[test]
fn headless_host_controller_cancel_is_sticky_across_wakes_and_waits() {
    let controller = HeadlessController::default();
    let waiting = controller.clone();
    let (started_tx, started_rx) = std::sync::mpsc::sync_channel(0);
    let worker = std::thread::spawn(move || {
        started_tx.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            if waiting.wait_event(deadline) == HeadlessWait::Cancelled {
                return true;
            }
        }
    });
    started_rx.recv().unwrap();
    controller.wake();
    controller.cancel();
    controller.cancel();
    assert!(worker.join().unwrap());
    assert_eq!(
        controller.wait_event(Instant::now()),
        HeadlessWait::Cancelled
    );
}

#[test]
fn headless_host_early_and_repeated_wakes_coalesce_without_changing_deadline() {
    let controller = HeadlessController::default();
    controller.wake();
    controller.wake();
    let deadline = Instant::now() + Duration::from_secs(60);
    assert_eq!(controller.wait_event(deadline), HeadlessWait::Woken);
    controller.wake();
    assert_eq!(
        controller.wait_event(Instant::now()),
        HeadlessWait::Deadline
    );
    controller.cancel();
    assert_eq!(controller.wait_event(deadline), HeadlessWait::Cancelled);
}

#[test]
fn headless_host_schedule_preserves_cadence_and_bounds_overload() {
    let mut schedule = HeadlessSchedule::new(Duration::from_millis(10)).unwrap();
    schedule.tick_completed(Duration::from_millis(3)).unwrap();
    assert_eq!(schedule.next_tick(), Duration::from_millis(10));
    schedule.tick_completed(Duration::from_millis(12)).unwrap();
    assert_eq!(schedule.next_tick(), Duration::from_millis(20));
    schedule.tick_completed(Duration::from_millis(100)).unwrap();
    assert_eq!(schedule.next_tick(), Duration::from_millis(110));
    assert_eq!(schedule.overruns(), 1);
    assert!(schedule.tick_completed(Duration::from_millis(99)).is_err());
}

#[test]
fn headless_host_schedule_rejects_zero_step_and_overflow() {
    assert!(HeadlessSchedule::new(Duration::ZERO).is_err());
    let mut schedule = HeadlessSchedule::new(Duration::from_secs(1)).unwrap();
    assert!(schedule.tick_completed(Duration::MAX).is_err());
}

#[test]
fn headless_host_args_reuse_profile_parser_and_reject_conflicts() {
    let parsed = args::parse(["--ticks=3", "--runtime-session-profile=headless"]).unwrap();
    assert_eq!(parsed.tick_limit.unwrap().get(), 3);
    for args in [
        vec!["--ticks=0"],
        vec!["--ticks=1", "--ticks=2"],
        vec!["--ticks=-1"],
        vec!["--ticks=1.0"],
        vec!["--runtime-session-profile=runtime"],
        vec!["--unknown"],
        vec!["--play-scene=scene.json"],
    ] {
        assert!(args::parse(args).is_err());
    }
    assert!(args::parse(["--help"]).unwrap().help);
}

#[test]
fn headless_host_help_does_not_enter_runtime_startup() {
    let report =
        super::EntryRunner::run_headless_with_args(["--help"], HeadlessController::default())
            .unwrap();
    assert_eq!(report.reason, HeadlessStopReason::Help);
    assert!(!report.ready);
}

#[test]
fn headless_host_cancel_before_start_does_not_load_runtime() {
    let controller = HeadlessController::default();
    controller.cancel();
    let report = super::EntryRunner::run_headless_with_args(["--ticks=2"], controller).unwrap();
    assert_eq!(report.reason, HeadlessStopReason::Cancelled);
    assert_eq!(report.completed_ticks, 0);
    assert!(!report.ready);
}

#[test]
fn headless_host_mid_start_cancel_returns_clean_report_without_retaining_owner() {
    let controller = HeadlessController::default();
    let managed_controller = controller.clone();
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::sync_channel(1);
    let (failures_tx, failures_rx) = std::sync::mpsc::sync_channel(1);
    let managed = std::thread::spawn(move || {
        super::managed::run_owned(&managed_controller, move |owner, watch| {
            failures_tx.send(watch.failures.clone()).unwrap();
            let startup = {
                let _operation = watch.operation("startup", Duration::from_secs(30));
                started_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                super::session::ensure_starting(&owner).map(|()| FakeSession::default())
            };
            super::session::finish_startup(
                startup,
                &owner,
                watch,
                NonZeroU64::new(1),
                Duration::from_nanos(1),
            )
        })
    });

    started_rx.recv().unwrap();
    let failures = failures_rx.recv().unwrap();
    controller.cancel();
    release_tx.send(()).unwrap();

    let report = managed.join().unwrap().unwrap();
    assert_eq!(report.reason, HeadlessStopReason::Cancelled);
    assert_eq!(report.completed_ticks, 0);
    assert!(!report.ready);
    assert!(failures.is_empty());
    assert!(controller.finish_runtime_until(Instant::now()));
    assert_eq!(controller.runtime_destroy_receipt(), None);
}

#[test]
fn headless_host_startup_failure_remains_terminal_during_cancellation() {
    let controller = HeadlessController::default();
    let (failures_tx, failures_rx) = std::sync::mpsc::sync_channel(1);
    let result = super::managed::run_owned(&controller, move |owner, watch| {
        failures_tx.send(watch.failures.clone()).unwrap();
        owner.cancel();
        super::session::finish_startup::<FakeSession>(
            Err(HeadlessHostError::Startup(
                "fixture startup failure".to_owned(),
            )),
            &owner,
            watch,
            NonZeroU64::new(1),
            Duration::from_nanos(1),
        )
    });

    let failures = failures_rx.recv().unwrap();
    assert!(matches!(
        result,
        Err(HeadlessHostError::Startup(detail)) if detail == "fixture startup failure"
    ));
    assert!(failures
        .snapshot()
        .primary()
        .unwrap()
        .message()
        .contains("fixture startup failure"));
    assert!(controller.finish_runtime_until(Instant::now()));
    assert!(controller.is_cancelled());
}

#[test]
fn headless_host_entry_source_keeps_desktop_event_loop_out_of_server_path() {
    let source = include_str!("../../headless.rs");
    assert!(!source.contains("winit::"));
    let wake = include_str!("../../../runtime_library/wake_registry.rs");
    assert!(wake.contains("register_callback"));
}

#[test]
fn headless_host_ready_requires_composition_and_first_host_drain() {
    for (fail_composition, fail_drain) in [(true, false), (false, true)] {
        let mut session = FakeSession {
            fail_composition,
            fail_drain,
            ..Default::default()
        };
        let failures = ProductFailureLedger::default();
        assert!(drive(
            &mut session,
            &HeadlessController::default(),
            NonZeroU64::new(1),
            Duration::from_nanos(1),
            &failures
        )
        .is_err());
        assert_eq!(session.ticks, u64::from(!fail_composition));
        assert_eq!(session.stops, 1);
    }
}

#[test]
fn headless_host_hung_operation_deadline_retains_owner_until_destroy_receipt() {
    for operation in ["startup", "tick", "host-drain"] {
        let controller = HeadlessController::default();
        let (release_tx, release_rx) = std::sync::mpsc::sync_channel(1);
        let result = super::managed::run_owned(&controller, move |_, watch| {
            let call = watch.operation(operation, Duration::from_millis(5));
            release_rx.recv().unwrap();
            drop(call);
            watch.record_destroy(&Ok(()));
            Ok(super::session::HeadlessRunReport::help())
        });
        let retained = result
            .as_ref()
            .err()
            .and_then(HeadlessHostError::retained_controller)
            .is_some();
        let still_running = !controller.finish_runtime_until(Instant::now());
        let receipt_pending = controller.runtime_destroy_receipt().is_none();
        release_tx.send(()).unwrap();
        assert!(controller.finish_runtime_until(Instant::now() + Duration::from_secs(1)));
        assert!(
            matches!(result, Err(HeadlessHostError::RetainedOwner { cause, .. }) if matches!(*cause, HeadlessHostError::OperationDeadline(name) if name == operation))
        );
        assert!(controller.is_cancelled());
        assert!(retained && still_running && receipt_pending);
        assert_eq!(controller.runtime_destroy_receipt(), Some(Ok(())));
    }
}

#[test]
fn headless_host_hung_tick_cancellation_is_bounded_and_owner_can_be_reaped() {
    let controller = HeadlessController::default();
    let (release_tx, release_rx) = std::sync::mpsc::sync_channel(1);
    let result = super::managed::run_owned_with_grace(
        &controller,
        Duration::from_millis(5),
        move |controller, watch| {
            let call = watch.operation("tick", Duration::from_secs(60));
            controller.cancel();
            release_rx.recv().unwrap();
            drop(call);
            watch.record_destroy(&Ok(()));
            Ok(super::session::HeadlessRunReport::help())
        },
    );
    let retained_controller = result
        .as_ref()
        .err()
        .and_then(HeadlessHostError::retained_controller)
        .cloned();
    drop(controller);
    let still_running = retained_controller
        .as_ref()
        .is_some_and(|controller| !controller.finish_runtime_until(Instant::now()));
    release_tx.send(()).unwrap();
    assert!(retained_controller
        .unwrap()
        .finish_runtime_until(Instant::now() + Duration::from_secs(1)));
    assert!(
        matches!(result, Err(HeadlessHostError::RetainedOwner { cause, .. }) if matches!(*cause, HeadlessHostError::OperationCancelled("tick")))
    );
    assert!(still_running);
}

#[test]
fn headless_host_console_close_handshake_waits_for_full_shutdown_and_is_bounded() {
    let controller = HeadlessController::default();
    controller.cancel();
    assert!(!controller.wait_shutdown_until(Instant::now()));
    assert!(controller.shutdown_deadline_expired());
    controller.mark_shutdown_complete();
    assert!(controller.wait_shutdown_until(Instant::now()));
    assert!(controller.is_cancelled());
}

#[test]
fn headless_host_shutdown_phases_cannot_extend_the_first_deadline() {
    for cancel_first in [false, true] {
        let controller = HeadlessController::default();
        if cancel_first {
            controller.cancel();
        }
        let deadline = controller.begin_shutdown();
        controller.cancel();
        assert_eq!(
            controller.begin_shutdown_until(deadline + Duration::from_secs(1)),
            deadline
        );
        assert_eq!(controller.begin_shutdown(), deadline);
    }
}

#[test]
fn headless_host_terminal_log_deadline_is_visible_without_a_console_waiter() {
    let controller = HeadlessController::default();
    let deadline = controller.begin_shutdown_until(Instant::now());
    assert!(controller.finish_runtime_until(deadline));
    assert!(deadline.saturating_duration_since(Instant::now()).is_zero());
    controller.mark_shutdown_complete();
    assert!(controller.shutdown_deadline_expired());
    assert!(controller.wait_shutdown_until(deadline + Duration::from_secs(1)));
    assert!(controller.shutdown_deadline_expired());
}

#[test]
#[cfg(windows)]
fn headless_host_thread_exit_deadline_includes_tls_destructors() {
    let controller = HeadlessController::default();
    let (entered_tx, entered_rx) = std::sync::mpsc::sync_channel(1);
    let (release_tx, release_rx) = std::sync::mpsc::sync_channel(1);
    let started_at = Instant::now();
    let result = super::managed::run_owned_with_grace(
        &controller,
        Duration::from_millis(5),
        move |_, watch| {
            THREAD_EXIT_LATCH.with(|latch| {
                *latch.borrow_mut() = Some(ThreadExitLatch {
                    entered: entered_tx,
                    release: release_rx,
                })
            });
            watch.record_destroy(&Ok(()));
            Ok(super::session::HeadlessRunReport::help())
        },
    );
    let elapsed = started_at.elapsed();
    let entered = entered_rx.recv_timeout(Duration::from_secs(1)).is_ok();
    let still_running = !controller.finish_runtime_until(Instant::now());
    let _ = release_tx.send(());
    assert!(controller.finish_runtime_until(Instant::now() + Duration::from_secs(1)));
    assert!(elapsed < Duration::from_secs(1));
    assert!(entered && still_running);
    assert!(
        matches!(result, Err(HeadlessHostError::RetainedOwner { cause, .. }) if matches!(*cause, HeadlessHostError::OperationDeadline("thread-exit")))
    );
}

#[test]
fn headless_host_destroy_failure_retries_until_the_single_shutdown_deadline() {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    struct RetriedSession {
        inner: FakeSession,
        attempts: Arc<AtomicU64>,
        drops: Arc<AtomicU64>,
        owner_thread: std::thread::ThreadId,
        _local: std::rc::Rc<()>,
    }
    impl HeadlessSession for RetriedSession {
        fn verify_composition(&self) -> Result<(), HeadlessHostError> {
            self.inner.verify_composition()
        }
        fn tick(&mut self) -> Result<(), HeadlessHostError> {
            self.inner.tick()
        }
        fn drain_host(&mut self) -> Result<(), HeadlessHostError> {
            self.inner.drain_host()
        }
        fn stop(&mut self) -> Result<(), HeadlessHostError> {
            assert_eq!(self.owner_thread, std::thread::current().id());
            self.attempts.fetch_add(1, Ordering::SeqCst);
            self.inner.stop()
        }
    }
    impl Drop for RetriedSession {
        fn drop(&mut self) {
            assert_eq!(self.owner_thread, std::thread::current().id());
            assert!(self.inner.stops > self.inner.stop_failures);
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    for failures_before_success in [1, 2] {
        let controller = HeadlessController::default();
        let attempts = Arc::new(AtomicU64::new(0));
        let drops = Arc::new(AtomicU64::new(0));
        let worker_attempts = attempts.clone();
        let worker_drops = drops.clone();
        let result = super::managed::run_owned(&controller, move |controller, watch| {
            let mut session = super::session::ObservedSession::new(
                RetriedSession {
                    inner: FakeSession {
                        stop_failures: failures_before_success,
                        ..Default::default()
                    },
                    attempts: worker_attempts,
                    drops: worker_drops,
                    owner_thread: std::thread::current().id(),
                    _local: std::rc::Rc::new(()),
                },
                watch.clone(),
            );
            let result = drive(
                &mut session,
                &controller,
                NonZeroU64::new(1),
                Duration::from_nanos(1),
                &watch.failures,
            );
            session.finish_cleanup_on_owner();
            result
        });
        let retained = result
            .as_ref()
            .err()
            .and_then(HeadlessHostError::retained_controller)
            .is_some();
        let no_eager_drop = drops.load(Ordering::SeqCst) == 0;
        let initial_attempts = attempts.load(Ordering::SeqCst);
        let joined = controller.finish_runtime_until(Instant::now() + Duration::from_secs(1));
        assert!(joined && retained && no_eager_drop);
        assert_eq!(initial_attempts, 1);
        assert_eq!(attempts.load(Ordering::SeqCst), failures_before_success + 1);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert_eq!(controller.runtime_destroy_receipt(), Some(Ok(())));
    }
}

#[test]
#[ignore = "requires a staged matching headless Runtime DLL BuildSet"]
fn headless_host_real_dll_runs_two_ticks_and_reports_destroy_receipt() {
    let controller = HeadlessController::default();
    let result = super::EntryRunner::run_headless_with_args(["--ticks=2"], controller.clone());
    let joined = controller.finish_runtime_until(Instant::now() + Duration::from_secs(3));
    let report = result.unwrap();
    assert!(joined && report.ready);
    assert_eq!(report.completed_ticks, 2);
    assert_eq!(controller.runtime_destroy_receipt(), Some(Ok(())));
}
