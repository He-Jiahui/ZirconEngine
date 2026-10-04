use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use zircon_runtime_interface::ZrRuntimeOperationHandle;

use crate::operation::{RuntimeOperationApply, RuntimeOperationSnapshot};

use super::*;

const SNAPSHOT_OWNER_BYTES: usize = 128;
const TERMINAL_WAIT: Duration = Duration::from_secs(5);

#[derive(Default)]
struct SnapshotCounts {
    snapshots: AtomicUsize,
    prepares: AtomicUsize,
    applies: AtomicUsize,
    drops: AtomicUsize,
}

struct CountedSnapshotOwner {
    counts: Arc<SnapshotCounts>,
}

impl Drop for CountedSnapshotOwner {
    fn drop(&mut self) {
        self.counts.drops.fetch_add(1, Ordering::SeqCst);
    }
}

struct SnapshotBudgetHandler {
    counts: Arc<SnapshotCounts>,
    logical_owner_bytes: usize,
}

impl RuntimeOperationHandler for SnapshotBudgetHandler {
    fn snapshot(
        &self,
        _context: RuntimeOperationContext<'_>,
        _payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        Err(RuntimeOperationHandlerError::new(
            "snapshot budget fixture requires the owned snapshot route",
        ))
    }

    fn snapshot_owned(
        &self,
        _context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<RuntimeOperationSnapshot, RuntimeOperationHandlerError> {
        self.counts.snapshots.fetch_add(1, Ordering::SeqCst);
        // The declared estimate is independent of this small allocation, including
        // the overflow case. The production service accounts the reported bytes.
        Ok(RuntimeOperationSnapshot::with_owner_state(
            payload,
            Box::new(CountedSnapshotOwner {
                counts: Arc::clone(&self.counts),
            }),
            self.logical_owner_bytes,
        ))
    }

    fn prepare(
        &self,
        _snapshot: serde_json::Value,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        self.counts.prepares.fetch_add(1, Ordering::SeqCst);
        Err(RuntimeOperationHandlerError::new(
            "snapshot budget fixture must retain its owner through prepare",
        ))
    }

    fn prepare_owned(
        &self,
        snapshot: RuntimeOperationSnapshot,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        self.counts.prepares.fetch_add(1, Ordering::SeqCst);
        let (payload, owner, owner_bytes) = snapshot.into_parts();
        assert_eq!(payload, snapshot_payload());
        let owner = owner.ok_or_else(|| {
            RuntimeOperationHandlerError::new("snapshot budget fixture lost its owner")
        })?;
        // These two null values fit below the original payload's serialized size,
        // so a later result reservation cannot mask the snapshot boundary.
        Ok(RuntimeOperationPrepared::with_owner_state(
            serde_json::Value::Null,
            serde_json::Value::Null,
            owner,
            owner_bytes,
        ))
    }

    fn apply(
        &self,
        _context: RuntimeOperationContext<'_>,
        _command: serde_json::Value,
    ) -> Result<(), RuntimeOperationHandlerError> {
        self.counts.applies.fetch_add(1, Ordering::SeqCst);
        Err(RuntimeOperationHandlerError::new(
            "snapshot budget fixture must retain its owner through apply",
        ))
    }

    fn apply_owned(
        &self,
        _context: RuntimeOperationContext<'_>,
        apply: RuntimeOperationApply,
    ) -> Result<(), RuntimeOperationHandlerError> {
        self.counts.applies.fetch_add(1, Ordering::SeqCst);
        let (command, owner) = apply.into_parts();
        assert!(command.is_null());
        let owner = owner
            .ok_or_else(|| RuntimeOperationHandlerError::new("prepared owner is missing"))?
            .downcast::<CountedSnapshotOwner>()
            .map_err(|_| RuntimeOperationHandlerError::new("prepared owner type changed"))?;
        assert!(Arc::ptr_eq(&owner.counts, &self.counts));
        drop(owner);
        Ok(())
    }
}

fn snapshot_payload() -> serde_json::Value {
    serde_json::json!({"value": 7})
}

fn payload_bytes() -> usize {
    serde_json::to_vec(&snapshot_payload()).unwrap().len()
}

fn budget_service(max_retained_bytes: usize) -> RuntimeOperationService {
    RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 1,
        max_in_flight_prepares: 1,
        max_retained_bytes,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::from_secs(60),
    })
}

fn register_budget_handler(
    service: &mut RuntimeOperationService,
    operation_id: &str,
    logical_owner_bytes: usize,
    counts: &Arc<SnapshotCounts>,
) {
    service
        .register_handler(
            operation_id,
            Arc::new(SnapshotBudgetHandler {
                counts: Arc::clone(counts),
                logical_owner_bytes,
            }),
        )
        .unwrap();
}

fn submit_snapshot(
    service: &RuntimeOperationService,
    operation_id: &str,
) -> ZrRuntimeOperationHandle {
    service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            operation_id,
            snapshot_payload(),
        ))
        .unwrap()
}

fn snapshot_level() -> LevelSystem {
    LevelSystem::new(
        WorldHandle::new(1),
        Arc::new(Mutex::new(World::empty())),
        LevelMetadata::default(),
    )
}

fn drive_to_terminal(
    service: &RuntimeOperationService,
    runtime: &CoreRuntime,
    level: &LevelSystem,
    handle: ZrRuntimeOperationHandle,
) -> ZrRuntimeOperationPhase {
    let deadline = Instant::now() + TERMINAL_WAIT;
    loop {
        service.tick(&runtime.handle(), level);
        let phase = service.poll(handle).unwrap().phase().unwrap();
        if phase.is_terminal() {
            return phase;
        }
        assert!(
            Instant::now() < deadline,
            "owned snapshot did not reach a terminal phase before its deadline"
        );
        std::thread::yield_now();
    }
}

fn assert_counts(counts: &SnapshotCounts, prepares: usize, applies: usize) {
    assert_eq!(counts.snapshots.load(Ordering::SeqCst), 1);
    assert_eq!(counts.prepares.load(Ordering::SeqCst), prepares);
    assert_eq!(counts.applies.load(Ordering::SeqCst), applies);
    assert_eq!(counts.drops.load(Ordering::SeqCst), 1);
}

fn assert_snapshot_rejected(
    service: &RuntimeOperationService,
    runtime: &CoreRuntime,
    level: &LevelSystem,
    handle: ZrRuntimeOperationHandle,
    counts: &SnapshotCounts,
) {
    // Snapshot admission is synchronous in this owner tick: no worker is needed
    // to observe rejection or the owned state's destruction.
    service.tick(&runtime.handle(), level);
    assert_eq!(
        service.poll(handle).unwrap().phase(),
        Some(ZrRuntimeOperationPhase::Failed)
    );
    assert_counts(counts, 0, 0);
    assert_eq!(service.lock_state().in_flight_prepares, 0);
    let result = service.harvest(handle).unwrap();
    assert!(result.failure().is_some_and(|error| {
        error.contains("immutable snapshot exceeds retained byte budget")
    }));
    assert_eq!(service.lock_state().retained_bytes, 0);
    service.tick(&runtime.handle(), level);
    assert_counts(counts, 0, 0);
    assert_eq!(service.lock_state().in_flight_prepares, 0);
    assert_eq!(service.lock_state().retained_bytes, 0);
}

#[test]
fn snapshot_owner_exact_budget_dispatches_prepare_and_apply() {
    let counts = Arc::new(SnapshotCounts::default());
    let mut service = budget_service(payload_bytes() + SNAPSHOT_OWNER_BYTES);
    register_budget_handler(
        &mut service,
        "test.snapshot.exact",
        SNAPSHOT_OWNER_BYTES,
        &counts,
    );
    let handle = submit_snapshot(&service, "test.snapshot.exact");
    assert_eq!(service.lock_state().retained_bytes, payload_bytes());
    let runtime = CoreRuntime::new();
    let level = snapshot_level();

    assert_eq!(
        drive_to_terminal(&service, &runtime, &level, handle),
        ZrRuntimeOperationPhase::Completed
    );
    assert_counts(&counts, 1, 1);
    assert_eq!(service.lock_state().in_flight_prepares, 0);
    assert!(service
        .harvest(handle)
        .unwrap()
        .succeeded_output()
        .unwrap()
        .is_null());
    assert_eq!(service.lock_state().retained_bytes, 0);
}

#[test]
fn snapshot_owner_over_budget_fails_before_prepare_and_reuses_capacity_after_harvest() {
    let rejected_counts = Arc::new(SnapshotCounts::default());
    let mut service = budget_service(payload_bytes() + SNAPSHOT_OWNER_BYTES - 1);
    register_budget_handler(
        &mut service,
        "test.snapshot.too-large",
        SNAPSHOT_OWNER_BYTES,
        &rejected_counts,
    );
    let handle = submit_snapshot(&service, "test.snapshot.too-large");
    let runtime = CoreRuntime::new();
    let level = snapshot_level();
    assert_snapshot_rejected(&service, &runtime, &level, handle, &rejected_counts);

    let replacement_counts = Arc::new(SnapshotCounts::default());
    register_budget_handler(
        &mut service,
        "test.snapshot.reuse",
        std::mem::size_of::<CountedSnapshotOwner>(),
        &replacement_counts,
    );
    let replacement = submit_snapshot(&service, "test.snapshot.reuse");
    assert_eq!(
        drive_to_terminal(&service, &runtime, &level, replacement),
        ZrRuntimeOperationPhase::Completed
    );
    assert_counts(&replacement_counts, 1, 1);
    assert_counts(&rejected_counts, 0, 0);
    assert!(service
        .harvest(replacement)
        .unwrap()
        .succeeded_output()
        .unwrap()
        .is_null());
    assert_eq!(service.lock_state().retained_bytes, 0);
    assert_eq!(service.lock_state().in_flight_prepares, 0);
}

#[test]
fn snapshot_owner_checked_add_overflow_fails_without_large_allocation() {
    let counts = Arc::new(SnapshotCounts::default());
    let mut service = budget_service(1_024);
    register_budget_handler(&mut service, "test.snapshot.overflow", usize::MAX, &counts);
    let handle = submit_snapshot(&service, "test.snapshot.overflow");
    assert!(service.lock_state().retained_bytes > 0);
    let runtime = CoreRuntime::new();
    let level = snapshot_level();

    assert_snapshot_rejected(&service, &runtime, &level, handle, &counts);
}
