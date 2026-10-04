//! Windows-only test scheduling/data for one real public startup attempt.

use std::cell::RefCell;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, ThreadId};
use std::time::Instant;

use crate::dynamic_api::session::{RuntimeDynamicSessionError, RuntimeProjectError};
use zircon_runtime_interface::ZrRuntimeSessionHandle;

use super::runtime::startup_observation::RuntimeStartupObservation;
use super::{CleanupReceipt, OwnerShutdownReceipt, RECEIPT_PENDING, RECEIPT_SUCCEEDED};

const MAX_WITNESS_BYTES: usize = 32 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Primary {
    allocated_handle: u64,
    diagnostic: String,
    step: &'static str,
    read_path: PathBuf,
    read_source: String,
    read_source_debug: String,
    read_os_error: Option<i32>,
    read_kind: std::io::ErrorKind,
}

#[derive(Default)]
struct State {
    owner: usize,
    receipt: usize,
    worker: Option<ThreadId>,
    deadline: Option<Instant>,
    primary: Option<Primary>,
    primary_records: usize,
    admitted: usize,
    callback_entries: usize,
    callback_completions: usize,
    callback_completed: bool,
    publisher_locked: bool,
    would_block: usize,
    would_block_state: Option<u8>,
    release_publisher: bool,
    published_state: Option<u8>,
    terminal: bool,
    tls_entered: bool,
    release_tls: bool,
    tls_completed: bool,
    tls_worker: Option<ThreadId>,
    owner_receipt: Option<OwnerShutdownReceipt>,
    core: Option<serde_json::Value>,
}

pub(super) struct Schedule {
    nonce: String,
    play_path: PathBuf,
    witness: PathBuf,
    state: Mutex<State>,
    changed: Condvar,
}

pub(super) type Publication = Arc<Schedule>;

thread_local! {
    static CURRENT: RefCell<Option<Publication>> = const { RefCell::new(None) };
    static EXIT_LATCH: RefCell<Option<WorkerExit>> = const { RefCell::new(None) };
}

pub(super) struct RestoreCurrent(Option<Publication>);

impl Drop for RestoreCurrent {
    fn drop(&mut self) {
        CURRENT.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}

fn wait_engine(schedule: &Publication, label: &str, ready: impl Fn(&State) -> bool) {
    let mut state = schedule.state.lock().expect("publication schedule");
    while !ready(&state) {
        let deadline = state.deadline.expect("original engine deadline");
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(!remaining.is_zero(), "original engine deadline: {label}");
        let (next, timeout) = schedule
            .changed
            .wait_timeout(state, remaining)
            .expect("publication schedule wait");
        state = next;
        assert!(
            !timeout.timed_out() || ready(&state),
            "engine wait: {label}"
        );
    }
}

fn assert_worker(state: &State) {
    assert_eq!(state.worker, Some(thread::current().id()));
}

pub(super) fn current() -> Option<Publication> {
    CURRENT.with(|slot| slot.borrow().clone())
}

pub(super) fn bind_owner(identity: usize) -> Option<Publication> {
    let schedule = current()?;
    let mut state = schedule.state.lock().expect("publication schedule");
    assert_eq!(
        state.owner, 0,
        "one actual SessionOwner per failing attempt"
    );
    assert_ne!(identity, 0);
    state.owner = identity;
    drop(state);
    Some(schedule)
}

pub(super) fn worker_enter(schedule: &Option<Publication>) -> Option<RestoreCurrent> {
    let schedule = schedule.as_ref()?;
    let worker = thread::current().id();
    let previous = CURRENT.with(|slot| {
        let mut slot = slot.borrow_mut();
        assert!(
            slot.is_none(),
            "this worker has no other publication schedule"
        );
        slot.replace(Arc::clone(schedule))
    });
    schedule.state.lock().expect("publication schedule").worker = Some(worker);
    EXIT_LATCH.with(|slot| {
        let mut slot = slot.borrow_mut();
        assert!(slot.is_none());
        *slot = Some(WorkerExit {
            schedule: Arc::clone(schedule),
            worker,
        });
    });
    Some(RestoreCurrent(previous))
}

struct WorkerExit {
    schedule: Publication,
    worker: ThreadId,
}

impl Drop for WorkerExit {
    fn drop(&mut self) {
        // This value is never extracted: only the actual worker's Windows TLS exit drops it.
        {
            let mut state = self.schedule.state.lock().expect("publication schedule");
            assert!(
                state.terminal,
                "actual owner task terminal precedes worker TLS exit"
            );
            assert_eq!(state.worker, Some(self.worker));
            state.tls_worker = Some(self.worker);
            state.tls_entered = true;
            self.schedule.changed.notify_all();
        }
        wait_engine(&self.schedule, "same worker TLS release", |state| {
            state.release_tls
        });
        self.schedule
            .state
            .lock()
            .expect("publication schedule")
            .tls_completed = true;
        self.schedule.changed.notify_all();
    }
}

pub(super) fn startup_deadline(deadline: Instant) {
    if let Some(schedule) = current() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert!(state.deadline.replace(deadline).is_none());
        schedule.changed.notify_all();
    }
}

pub(super) fn prepare_receipt(receipt: &CleanupReceipt, owner: usize, deadline: Instant) {
    if let Some(schedule) = receipt.publication.as_ref() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_eq!(state.owner, owner);
        assert_eq!(state.deadline, Some(deadline));
        assert_eq!(state.receipt, 0);
        state.receipt = receipt as *const CleanupReceipt as usize;
    }
}

pub(super) fn admitted(receipt: &CleanupReceipt) {
    if let Some(schedule) = receipt.publication.as_ref() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_eq!(state.receipt, receipt as *const CleanupReceipt as usize);
        state.admitted += 1;
        assert_eq!(state.admitted, 1);
        schedule.changed.notify_all();
    }
}

pub(super) fn callback_entered(deadline: Instant) {
    if let Some(schedule) = current() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_worker(&state);
        assert_eq!(state.deadline, Some(deadline));
        state.callback_entries += 1;
        assert_eq!(state.callback_entries, 1);
    }
}

pub(super) fn callback_completed(deadline: Instant, completed: bool) {
    if let Some(schedule) = current() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_worker(&state);
        assert_eq!(state.deadline, Some(deadline));
        state.callback_completions += 1;
        assert_eq!(state.callback_completions, 1);
        state.callback_completed = completed;
    }
}

pub(super) fn publisher_locked(receipt: &CleanupReceipt, value: u8) {
    if let Some(schedule) = receipt.publication.as_ref() {
        {
            let mut state = schedule.state.lock().expect("publication schedule");
            assert_worker(&state);
            assert_eq!(state.receipt, receipt as *const CleanupReceipt as usize);
            assert_eq!(
                value, RECEIPT_SUCCEEDED,
                "this real callback completed successfully"
            );
            assert_eq!(state.callback_completions, 1);
            assert!(state.callback_completed);
            state.publisher_locked = true;
            schedule.changed.notify_all();
        }
        // The caller still holds the actual CleanupReceipt::wait Mutex, before its store.
        wait_engine(schedule, "release original receipt publisher", |state| {
            state.release_publisher
        });
    }
}

pub(super) fn before_wait(receipt: &CleanupReceipt, deadline: Instant) {
    if let Some(schedule) = receipt.publication.as_ref() {
        assert_eq!(
            schedule
                .state
                .lock()
                .expect("publication schedule")
                .deadline,
            Some(deadline)
        );
        wait_engine(
            schedule,
            "actual publisher owns the receipt Mutex",
            |state| state.publisher_locked,
        );
    }
}

pub(super) fn actual_would_block(receipt: &CleanupReceipt, deadline: Instant) {
    if let Some(schedule) = receipt.publication.as_ref() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_eq!(state.deadline, Some(deadline));
        assert_eq!(state.receipt, receipt as *const CleanupReceipt as usize);
        assert!(state.publisher_locked);
        let actual = receipt.state.load(std::sync::atomic::Ordering::Acquire);
        assert_eq!(actual, RECEIPT_PENDING, "publish has not stored success");
        state.would_block += 1;
        state.would_block_state = Some(actual);
        schedule.changed.notify_all();
    }
}

pub(super) fn published(receipt: &CleanupReceipt) {
    if let Some(schedule) = receipt.publication.as_ref() {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_worker(&state);
        state.published_state = Some(receipt.state.load(std::sync::atomic::Ordering::Acquire));
        schedule.changed.notify_all();
    }
}

pub(super) fn terminal_published(schedule: &Option<Publication>, terminal: bool) {
    if let Some(schedule) = schedule {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_worker(&state);
        assert!(terminal);
        state.terminal = terminal;
        schedule.changed.notify_all();
    }
}

pub(super) fn record_primary(primary: &RuntimeDynamicSessionError, allocated_handle: u64) {
    let Some(schedule) = current() else {
        return;
    };
    let RuntimeDynamicSessionError::ProjectStep {
        step: "load Play scene override",
        source: RuntimeProjectError::ReadPlayScene { path, source },
    } = primary
    else {
        panic!("expected the original typed ordinary ReadPlayScene error: {primary:?}");
    };
    assert_eq!(source.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(
        std::fs::canonicalize(path).expect("actual read path"),
        schedule.play_path
    );
    let captured = Primary {
        allocated_handle,
        diagnostic: primary.to_string(),
        step: "load Play scene override",
        read_path: path.clone(),
        read_source: source.to_string(),
        read_source_debug: format!("{source:?}"),
        read_os_error: source.raw_os_error(),
        read_kind: source.kind(),
    };
    let mut state = schedule.state.lock().expect("publication schedule");
    assert_worker(&state);
    if let Some(original) = state.primary.as_ref() {
        assert_eq!(
            original, &captured,
            "cleanup must preserve the first complete typed primary"
        );
    } else {
        state.primary = Some(captured);
    }
    state.primary_records += 1;
    assert!(state.primary_records <= 2, "factory and one cleanup only");
}

fn snapshot(schedule: &Publication, public_call_returned: bool) -> serde_json::Value {
    let state = schedule.state.lock().expect("publication schedule");
    let primary = state.primary.as_ref().expect("first typed primary");
    serde_json::json!({
        "nonce": schedule.nonce, "owner": state.owner, "receipt": state.receipt,
        "worker": format!("{:?}", state.worker.expect("actual worker")),
        "primary": {"allocated_handle": primary.allocated_handle, "diagnostic": primary.diagnostic,
            "variant": "ProjectStep/ReadPlayScene", "step": primary.step,
            "read_path": primary.read_path, "read_source": primary.read_source,
            "read_source_debug": primary.read_source_debug, "read_os_error": primary.read_os_error,
            "read_kind": format!("{:?}", primary.read_kind), "records": state.primary_records},
        "same_original_deadline": state.deadline.is_some(), "admitted": state.admitted,
        "callback_entries": state.callback_entries, "callback_completions": state.callback_completions,
        "callback_completed": state.callback_completed, "publisher_locked": state.publisher_locked,
        "actual_would_block": state.would_block, "would_block_state": state.would_block_state,
        "published_state": state.published_state, "actual_terminal": state.terminal,
        "tls_entered": state.tls_entered, "tls_completed": state.tls_completed,
        "tls_same_worker": state.tls_worker == state.worker && state.tls_worker.is_some(),
        "owner_receipt": format!("{:?}", state.owner_receipt.expect("actual shutdown return")),
        "allocated_handle_published": if public_call_returned { Some(crate::dynamic_api::session::registry::session_handle_is_published_for_test(ZrRuntimeSessionHandle::new(primary.allocated_handle))) } else { None },
        "actual_core": state.core,
    })
}

pub(super) fn owner_returned(
    receipt: OwnerShutdownReceipt,
    observation: &RuntimeStartupObservation,
) {
    let Some(schedule) = current() else {
        return;
    };
    {
        let mut state = schedule.state.lock().expect("publication schedule");
        assert_eq!(state.primary_records, 2);
        assert_eq!(
            state.primary.as_ref().expect("first primary").diagnostic,
            observation
                .primary_diagnostic
                .as_ref()
                .expect("original diagnostic")
                .as_str()
        );
        state.owner_receipt = Some(receipt);
        state.core = Some(serde_json::json!({
            "identity": observation.core_identity, "registered_modules": observation.registered_module_count,
            "all_modules_unloaded": observation.all_modules_unloaded, "pending": observation.core_pending,
            "cleanup_completed": observation.owner_cleanup_completed,
            "report_present": observation.core_shutdown_report.is_some(),
            "all_owned_graph_workers_joined": observation.core_shutdown_report.as_ref().is_some_and(|report| !report.has_in_flight_work() && report.timer_joined && report.scopes.iter().all(|scope| scope.is_quiescent()) && report.worker_shutdowns.len() == 3 && report.worker_shutdowns.iter().all(|workers| workers.expected_worker_count > 0 && workers.all_joined())),
        }));
    }
    // Never reenter the Store from its in-progress create callback. No slot result is claimed here.
    let raw = serde_json::to_vec(&snapshot(&schedule, false)).expect("bounded witness encoding");
    assert!(
        raw.len() <= MAX_WITNESS_BYTES,
        "bounded exact-child witness"
    );
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&schedule.witness)
        .expect("exclusive parent-owned witness");
    file.write_all(&raw)
        .expect("write exact pre-fatal/Joined witness");
    file.flush().expect("flush witness");
    file.sync_all()
        .expect("durable witness before existing fatal branch");
}

fn control(schedule: Publication, outer_deadline: Instant) {
    {
        let mut state = schedule.state.lock().expect("publication schedule");
        while state.deadline.is_none() {
            let remaining = outer_deadline.saturating_duration_since(Instant::now());
            assert!(
                !remaining.is_zero(),
                "outer child watchdog before engine shutdown starts"
            );
            let (next, timeout) = schedule
                .changed
                .wait_timeout(state, remaining)
                .expect("deadline attachment");
            state = next;
            assert!(!timeout.timed_out() || state.deadline.is_some());
        }
    }
    wait_engine(&schedule, "real Mutex contention branch", |state| {
        state.would_block > 0
    });
    schedule
        .state
        .lock()
        .expect("publication schedule")
        .release_publisher = true;
    schedule.changed.notify_all();
    wait_engine(&schedule, "actual worker TLS entry", |state| {
        state.tls_entered
    });
    schedule
        .state
        .lock()
        .expect("publication schedule")
        .release_tls = true;
    schedule.changed.notify_all();
    wait_engine(&schedule, "actual worker TLS completion", |state| {
        state.tls_completed
    });
}

pub(in crate::dynamic_api::session) fn with_schedule_for_test<R>(
    nonce: String,
    play_path: PathBuf,
    witness: PathBuf,
    outer_deadline: Instant,
    action: impl FnOnce() -> R,
) -> (R, serde_json::Value) {
    let schedule = Arc::new(Schedule {
        nonce,
        play_path,
        witness,
        state: Mutex::new(State::default()),
        changed: Condvar::new(),
    });
    let previous = CURRENT.with(|slot| {
        let mut slot = slot.borrow_mut();
        assert!(slot.is_none(), "one exact public failing call per schedule");
        slot.replace(Arc::clone(&schedule))
    });
    let restore = RestoreCurrent(previous);
    let for_control = Arc::clone(&schedule);
    let controller = thread::spawn(move || control(for_control, outer_deadline));
    let result = action();
    controller.join().expect("original-deadline controller");
    drop(restore);
    (result, snapshot(&schedule, true))
}
