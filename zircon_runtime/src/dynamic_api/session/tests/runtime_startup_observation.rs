use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use crate::core::{LifecycleState, TaskGraphShutdownReport};
use crate::dynamic_api::session::construction::RuntimeConstructionFailure;
use crate::dynamic_api::session::{RuntimeDynamicSessionError, RuntimeProjectError};

use super::{OwnerShutdownReceipt, RuntimeDynamicSession, RuntimeSessionStartupFailure};

#[derive(Clone, Debug, Default)]
pub(in crate::dynamic_api::session) struct RuntimeStartupObservation {
    pub allocated_handle: Option<u64>,
    pub primary_diagnostic: Option<String>,
    pub primary_step: Option<&'static str>,
    pub primary_read_error_kind: Option<std::io::ErrorKind>,
    pub core_identity: Option<usize>,
    pub registered_module_count: usize,
    pub all_modules_unloaded: Option<bool>,
    pub core_pending: Option<bool>,
    pub secondary_shutdown_diagnostic: Option<String>,
    pub core_shutdown_report: Option<TaskGraphShutdownReport>,
    pub owner_cleanup_completed: Option<bool>,
    pub owner_shutdown_receipt: Option<OwnerShutdownReceipt>,
}

#[cfg(windows)]
impl RuntimeStartupObservation {
    pub(in crate::dynamic_api::session) fn with_receipt_publication_for_test<R>(
        nonce: String,
        play_path: std::path::PathBuf,
        witness: std::path::PathBuf,
        outer_deadline: std::time::Instant,
        action: impl FnOnce() -> R,
    ) -> (R, serde_json::Value) {
        super::super::receipt_publication::with_schedule_for_test(
            nonce,
            play_path,
            witness,
            outer_deadline,
            action,
        )
    }
}

pub(super) type ObservationSink = Arc<Mutex<RuntimeStartupObservation>>;

thread_local! {
    static CURRENT: RefCell<Option<ObservationSink>> = RefCell::new(None);
}

struct RestoreObservation(Option<ObservationSink>);

impl Drop for RestoreObservation {
    fn drop(&mut self) {
        CURRENT.with(|current| *current.borrow_mut() = self.0.take());
    }
}

pub(in crate::dynamic_api::session) fn observe_runtime_startup_for_test<R>(
    action: impl FnOnce() -> R,
) -> (R, RuntimeStartupObservation) {
    let sink = Arc::new(Mutex::new(RuntimeStartupObservation::default()));
    let previous = CURRENT.with(|current| {
        let mut current = current.borrow_mut();
        assert!(current.is_none(), "one public startup call per observation");
        current.replace(Arc::clone(&sink))
    });
    let restore = RestoreObservation(previous);
    let result = action();
    drop(restore);
    let observation = sink.lock().expect("startup observation").clone();
    (result, observation)
}

pub(super) fn current() -> Option<ObservationSink> {
    CURRENT.with(|current| current.borrow().clone())
}

pub(in crate::dynamic_api::session) fn record_allocated_handle(handle: u64) {
    if let Some(sink) = current() {
        let mut observation = sink.lock().expect("startup observation");
        assert!(observation.allocated_handle.is_none());
        observation.allocated_handle = Some(handle);
    }
}

pub(super) fn record_failure(sink: &Option<ObservationSink>, failure: &RuntimeConstructionFailure) {
    let Some(sink) = sink else {
        return;
    };
    let mut observation = sink.lock().expect("startup observation");
    assert!(observation.allocated_handle.is_some());
    #[cfg(windows)]
    super::super::receipt_publication::record_primary(
        failure.primary(),
        observation
            .allocated_handle
            .expect("actual allocated handle"),
    );
    let diagnostic = failure.primary().to_string();
    let (step, read_kind) = match failure.primary() {
        RuntimeDynamicSessionError::ProjectStep { step, source } => (
            Some(*step),
            match source {
                RuntimeProjectError::ReadPlayScene { source, .. }
                | RuntimeProjectError::ReadNavmesh { source, .. } => Some(source.kind()),
                _ => None,
            },
        ),
        _ => (None, None),
    };
    if let Some(original) = observation.primary_diagnostic.as_ref() {
        assert_eq!(
            original, &diagnostic,
            "cleanup preserves the first primary diagnostic"
        );
        assert_eq!(observation.primary_step, step);
        assert_eq!(observation.primary_read_error_kind, read_kind);
    } else {
        observation.primary_diagnostic = Some(diagnostic);
        observation.primary_step = step;
        observation.primary_read_error_kind = read_kind;
    }
    observation.core_pending = Some(failure.has_pending_core());
    observation.secondary_shutdown_diagnostic = failure.shutdown_error().map(ToString::to_string);
    observation.core_shutdown_report = failure.shutdown_report().cloned();
    if let Some(core) = failure.observed_core.as_ref() {
        observation.core_identity = Some(Arc::as_ptr(&core.inner) as usize);
        let modules = core.inner.modules.lock().expect("observed Core modules");
        observation.registered_module_count = modules.len();
        observation.all_modules_unloaded = Some(
            modules
                .values()
                .all(|entry| entry.lifecycle == LifecycleState::Unloaded),
        );
    }
}

pub(super) fn record_cleanup(
    sink: &Option<ObservationSink>,
    state: &Result<RuntimeDynamicSession, RuntimeSessionStartupFailure>,
    completed: bool,
) {
    if let Err(failure) = state {
        record_failure(sink, &failure.construction);
        if let Some(sink) = sink {
            sink.lock()
                .expect("startup observation")
                .owner_cleanup_completed = Some(completed);
        }
    }
}

pub(super) fn record_owner_receipt(sink: &Option<ObservationSink>, receipt: OwnerShutdownReceipt) {
    if let Some(sink) = sink {
        let mut observation = sink.lock().expect("startup observation");
        observation.owner_shutdown_receipt = Some(receipt);
        #[cfg(windows)]
        super::super::receipt_publication::owner_returned(receipt, &observation);
    }
}
