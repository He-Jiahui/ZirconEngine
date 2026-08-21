use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use serde_json::json;
use zircon_runtime_interface::{
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationPhase,
    ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1,
};

mod phase_indexes;
mod source_guards;

use crate::core::runtime::CoreRuntime;
use crate::scene::World;

use super::service::RuntimeOperationLimits;
use super::{
    RuntimeOperationContext, RuntimeOperationHandler, RuntimeOperationHandlerError,
    RuntimeOperationPrepared, RuntimeOperationService, RuntimeOperationServiceError,
};

struct CountingHandler {
    preparations: Arc<AtomicUsize>,
    applications: Arc<AtomicUsize>,
}

struct PanicPrepareHandler;

struct PanicApplyHandler;

struct ResultReservationHandler {
    applications: Arc<AtomicUsize>,
}

impl RuntimeOperationHandler for PanicPrepareHandler {
    fn snapshot(
        &self,
        _context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        Ok(payload)
    }

    fn prepare(
        &self,
        _snapshot: serde_json::Value,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        panic!("prepare panic used to verify terminal containment")
    }

    fn apply(
        &self,
        _context: RuntimeOperationContext<'_>,
        _payload: serde_json::Value,
    ) -> Result<(), RuntimeOperationHandlerError> {
        panic!("failed preparation must never reach owner apply")
    }
}

impl RuntimeOperationHandler for CountingHandler {
    fn snapshot(
        &self,
        _context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        Ok(payload)
    }

    fn prepare(
        &self,
        snapshot: serde_json::Value,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        self.preparations.fetch_add(1, Ordering::SeqCst);
        Ok(RuntimeOperationPrepared::new(
            snapshot.clone(),
            json!({"echo": snapshot}),
        ))
    }

    fn apply(
        &self,
        _context: RuntimeOperationContext<'_>,
        _command: serde_json::Value,
    ) -> Result<(), RuntimeOperationHandlerError> {
        self.applications.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

impl RuntimeOperationHandler for PanicApplyHandler {
    fn snapshot(
        &self,
        _context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        Ok(payload)
    }

    fn prepare(
        &self,
        _snapshot: serde_json::Value,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        Ok(RuntimeOperationPrepared::new(
            serde_json::Value::Null,
            serde_json::json!({"unreachable": true}),
        ))
    }

    fn apply(
        &self,
        _context: RuntimeOperationContext<'_>,
        _command: serde_json::Value,
    ) -> Result<(), RuntimeOperationHandlerError> {
        panic!("owner apply panic used to verify terminal containment")
    }
}

impl RuntimeOperationHandler for ResultReservationHandler {
    fn snapshot(
        &self,
        _context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        Ok(payload)
    }

    fn prepare(
        &self,
        _snapshot: serde_json::Value,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        Ok(RuntimeOperationPrepared::new(
            serde_json::Value::Null,
            json!({"result": "x".repeat(256)}),
        ))
    }

    fn apply(
        &self,
        _context: RuntimeOperationContext<'_>,
        _command: serde_json::Value,
    ) -> Result<(), RuntimeOperationHandlerError> {
        self.applications.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn operation_service_dispatches_only_from_owner_tick() {
    let preparations = Arc::new(AtomicUsize::new(0));
    let applications = Arc::new(AtomicUsize::new(0));
    let mut service = RuntimeOperationService::new();
    service
        .register_handler(
            "test.echo",
            Arc::new(CountingHandler {
                preparations: preparations.clone(),
                applications: applications.clone(),
            }),
        )
        .unwrap();
    let request = ZrRuntimeOperationSubmitRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        "test.echo",
        json!({"value": 9}),
    );
    let handle = service.submit(request).unwrap();
    let runtime = CoreRuntime::new();
    let mut world = World::empty();

    let queued = service.poll(handle).unwrap();
    assert_eq!(queued.phase(), Some(ZrRuntimeOperationPhase::Queued));
    assert_eq!(preparations.load(Ordering::SeqCst), 0);
    assert_eq!(applications.load(Ordering::SeqCst), 0);

    let completed = tick_until_terminal(&service, &runtime, &mut world, handle);
    assert_eq!(completed.phase(), Some(ZrRuntimeOperationPhase::Completed));
    assert_eq!(preparations.load(Ordering::SeqCst), 1);
    assert_eq!(applications.load(Ordering::SeqCst), 1);

    let prepared = service
        .prepare_harvest(handle, |result| Ok::<_, ()>(result.clone()))
        .unwrap()
        .unwrap();
    assert_eq!(prepared.succeeded_output().unwrap()["echo"]["value"], 9);
    service.rollback_harvest(handle);
    assert_eq!(
        service.poll(handle).unwrap().phase(),
        Some(ZrRuntimeOperationPhase::Completed)
    );

    let result = service
        .prepare_harvest(handle, |result| Ok::<_, ()>(result.clone()))
        .unwrap()
        .unwrap();
    assert_eq!(result.succeeded_output().unwrap()["echo"]["value"], 9);
    service.commit_harvest(handle);
    assert!(matches!(
        service.harvest(handle),
        Err(RuntimeOperationServiceError::AlreadyHarvested { .. })
    ));
}

#[test]
fn operation_service_rejects_unknown_operations_duplicate_handlers_and_early_harvest() {
    let handler: Arc<dyn RuntimeOperationHandler> = Arc::new(CountingHandler {
        preparations: Arc::new(AtomicUsize::new(0)),
        applications: Arc::new(AtomicUsize::new(0)),
    });
    let mut service = RuntimeOperationService::new();
    service
        .register_handler("test.echo", handler.clone())
        .unwrap();
    assert!(matches!(
        service.register_handler("test.echo", handler),
        Err(RuntimeOperationServiceError::DuplicateHandler { .. })
    ));
    assert!(matches!(
        service.submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.missing",
            serde_json::Value::Null,
        )),
        Err(RuntimeOperationServiceError::UnknownOperation { .. })
    ));
    let handle = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.echo",
            serde_json::Value::Null,
        ))
        .unwrap();
    assert!(matches!(
        service.harvest(handle),
        Err(RuntimeOperationServiceError::NotTerminal { .. })
    ));
}

#[test]
fn operation_service_enforces_task_and_retained_byte_admission() {
    let handler: Arc<dyn RuntimeOperationHandler> = Arc::new(CountingHandler {
        preparations: Arc::new(AtomicUsize::new(0)),
        applications: Arc::new(AtomicUsize::new(0)),
    });
    let mut task_limited = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 1,
        max_in_flight_prepares: 1,
        max_retained_bytes: 64,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::from_secs(60),
    });
    task_limited
        .register_handler("test.echo", handler.clone())
        .unwrap();
    task_limited
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.echo",
            json!(1),
        ))
        .unwrap();
    assert!(matches!(
        task_limited.submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.echo",
            json!(2),
        )),
        Err(RuntimeOperationServiceError::TaskCapacityReached { maximum: 1 })
    ));

    let mut byte_limited = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 2,
        max_in_flight_prepares: 1,
        max_retained_bytes: 1,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::from_secs(60),
    });
    assert_eq!(byte_limited.max_retained_bytes(), 1);
    byte_limited.register_handler("test.echo", handler).unwrap();
    assert!(matches!(
        byte_limited.submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.echo",
            json!("too-large"),
        )),
        Err(RuntimeOperationServiceError::RetainedBytesCapacityReached { maximum: 1 })
    ));
}

#[test]
fn operation_service_reserves_raw_json_admission_before_decode_and_releases_rejections() {
    let handler: Arc<dyn RuntimeOperationHandler> = Arc::new(CountingHandler {
        preparations: Arc::new(AtomicUsize::new(0)),
        applications: Arc::new(AtomicUsize::new(0)),
    });
    let mut service = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 1,
        max_in_flight_prepares: 1,
        max_retained_bytes: 128,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::from_secs(60),
    });
    service.register_handler("test.echo", handler).unwrap();

    assert!(matches!(
        service.submit_json(br#"{"abi_version":1,"operation_id":"test.echo","payload":"#),
        Err(RuntimeOperationServiceError::InvalidRequest)
    ));
    let handle = service
        .submit_json(br#"{"abi_version":1,"operation_id":"test.echo","payload":null}"#)
        .expect("decode rejection releases the raw admission reservation");
    assert_eq!(
        service.poll(handle).unwrap().phase(),
        Some(ZrRuntimeOperationPhase::Queued)
    );

    let byte_limited = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 1,
        max_in_flight_prepares: 1,
        max_retained_bytes: 8,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::from_secs(60),
    });
    assert!(matches!(
        byte_limited.submit_json(b"{\"payload\":\"too-large\"}"),
        Err(RuntimeOperationServiceError::RetainedBytesCapacityReached { maximum: 8 })
    ));
}

#[test]
fn operation_service_reserves_prepared_result_before_owner_apply() {
    let applications = Arc::new(AtomicUsize::new(0));
    let mut service = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 1,
        max_in_flight_prepares: 1,
        max_retained_bytes: 128,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::from_secs(60),
    });
    service
        .register_handler(
            "test.result_reservation",
            Arc::new(ResultReservationHandler {
                applications: applications.clone(),
            }),
        )
        .unwrap();
    let handle = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.result_reservation",
            serde_json::Value::Null,
        ))
        .unwrap();
    let runtime = CoreRuntime::new();
    let mut world = World::empty();

    let terminal = tick_until_terminal(&service, &runtime, &mut world, handle);
    assert_eq!(terminal.phase(), Some(ZrRuntimeOperationPhase::Failed));
    assert_eq!(applications.load(Ordering::SeqCst), 0);
    assert!(
        service
            .harvest(handle)
            .unwrap()
            .failure()
            .is_some_and(|error| error.contains("prepared command and result exceed"))
    );
}

#[test]
fn operation_service_converts_prepare_panics_to_harvestable_failures() {
    let mut service = RuntimeOperationService::new();
    service
        .register_handler("test.panic", Arc::new(PanicPrepareHandler))
        .unwrap();
    let handle = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.panic",
            serde_json::Value::Null,
        ))
        .unwrap();
    let runtime = CoreRuntime::new();
    let mut world = World::empty();

    let terminal = tick_until_terminal(&service, &runtime, &mut world, handle);
    assert_eq!(terminal.phase(), Some(ZrRuntimeOperationPhase::Failed));
    assert!(
        service
            .harvest(handle)
            .unwrap()
            .failure()
            .is_some_and(|error| error.contains("prepare panicked"))
    );
}

#[test]
fn operation_service_converts_apply_panics_to_harvestable_failures() {
    let mut service = RuntimeOperationService::new();
    service
        .register_handler("test.apply_panic", Arc::new(PanicApplyHandler))
        .unwrap();
    let handle = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.apply_panic",
            serde_json::Value::Null,
        ))
        .unwrap();
    let runtime = CoreRuntime::new();
    let mut world = World::empty();

    let terminal = tick_until_terminal(&service, &runtime, &mut world, handle);
    assert_eq!(terminal.phase(), Some(ZrRuntimeOperationPhase::Failed));
    assert_eq!(
        terminal.detail_kind(),
        Some(ZrRuntimeOperationDetailKindV2::OwnerApplyFailed)
    );
    assert!(
        service
            .harvest(handle)
            .unwrap()
            .failure()
            .is_some_and(|error| error.contains("owner apply panicked"))
    );
}

fn tick_until_terminal(
    service: &RuntimeOperationService,
    runtime: &CoreRuntime,
    world: &mut World,
    handle: zircon_runtime_interface::ZrRuntimeOperationHandle,
) -> ZrRuntimeOperationStatusV2 {
    for _ in 0..1024 {
        service.tick(&runtime.handle(), world);
        let progress = service.poll(handle).unwrap();
        if progress
            .phase()
            .is_some_and(ZrRuntimeOperationPhase::is_terminal)
        {
            return progress;
        }
        std::thread::yield_now();
    }
    panic!("bounded operation task did not reach a terminal phase");
}

#[test]
fn operation_harvest_retains_a_zero_payload_status_tombstone() {
    let mut service = RuntimeOperationService::new();
    service
        .register_handler(
            "test.echo",
            Arc::new(CountingHandler {
                preparations: Arc::new(AtomicUsize::new(0)),
                applications: Arc::new(AtomicUsize::new(0)),
            }),
        )
        .unwrap();
    let handle = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.echo",
            serde_json::Value::Null,
        ))
        .unwrap();
    let runtime = CoreRuntime::new();
    let mut world = World::empty();

    let terminal = tick_until_terminal(&service, &runtime, &mut world, handle);
    assert_eq!(terminal.phase(), Some(ZrRuntimeOperationPhase::Completed));
    let _result = service
        .harvest(handle)
        .expect("completed result can harvest");

    let tombstone = service
        .poll(handle)
        .expect("harvested status remains observable");
    assert_eq!(tombstone.phase(), Some(ZrRuntimeOperationPhase::Harvested));
    assert_eq!(
        tombstone.detail_kind(),
        Some(ZrRuntimeOperationDetailKindV2::Harvested)
    );
    assert_eq!(
        tombstone.detail_value,
        ZrRuntimeOperationPhase::Completed.raw() as u64
    );
    assert!(matches!(
        service.harvest(handle),
        Err(RuntimeOperationServiceError::AlreadyHarvested { .. })
    ));
}

#[test]
fn terminal_harvest_updates_one_task_table_entry_to_a_bounded_tombstone() {
    let source = include_str!("service.rs");
    let start = source
        .find("    pub fn harvest(")
        .expect("operation harvest owner");
    let end = source[start..]
        .find("    fn arm_deadline(")
        .map(|offset| start + offset)
        .expect("operation deadline boundary");
    let harvest_source = &source[start..end];

    assert!(
        harvest_source.contains(".get_mut(&handle)"),
        "harvest must update its owned tombstone through one mutable lookup"
    );
    assert!(
        !harvest_source.contains("state.tasks.remove(&handle)"),
        "harvest must retain a bounded metadata tombstone rather than remove the task"
    );
}

#[test]
fn harvested_tombstones_evict_in_harvest_order_when_task_capacity_is_reused() {
    let mut service = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 1,
        max_in_flight_prepares: 1,
        max_retained_bytes: 1024,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::from_secs(60),
    });
    service
        .register_handler(
            "test.echo",
            Arc::new(CountingHandler {
                preparations: Arc::new(AtomicUsize::new(0)),
                applications: Arc::new(AtomicUsize::new(0)),
            }),
        )
        .unwrap();
    let runtime = CoreRuntime::new();
    let mut world = World::empty();
    let first = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.echo",
            serde_json::Value::Null,
        ))
        .unwrap();
    tick_until_terminal(&service, &runtime, &mut world, first);
    service.harvest(first).unwrap();
    assert_eq!(
        service.poll(first).unwrap().phase(),
        Some(ZrRuntimeOperationPhase::Harvested)
    );

    let second = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.echo",
            serde_json::Value::Null,
        ))
        .expect("oldest harvested tombstone makes room for a new task");

    assert!(matches!(
        service.poll(first),
        Err(RuntimeOperationServiceError::UnknownHandle { .. })
    ));
    assert_ne!(first, second);
}

#[test]
fn tombstone_eviction_uses_original_terminal_order_not_harvest_order() {
    let mut service = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 2,
        max_in_flight_prepares: 1,
        max_retained_bytes: 1024,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::from_secs(60),
    });
    service
        .register_handler(
            "test.echo",
            Arc::new(CountingHandler {
                preparations: Arc::new(AtomicUsize::new(0)),
                applications: Arc::new(AtomicUsize::new(0)),
            }),
        )
        .unwrap();
    let runtime = CoreRuntime::new();
    let mut world = World::empty();
    let first = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.echo",
            serde_json::Value::Null,
        ))
        .unwrap();
    tick_until_terminal(&service, &runtime, &mut world, first);
    let second = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.echo",
            serde_json::Value::Null,
        ))
        .unwrap();
    tick_until_terminal(&service, &runtime, &mut world, second);

    service.harvest(second).unwrap();
    service.harvest(first).unwrap();
    let third = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.echo",
            serde_json::Value::Null,
        ))
        .expect("earliest terminal tombstone makes room first");

    assert!(matches!(
        service.poll(first),
        Err(RuntimeOperationServiceError::UnknownHandle { .. })
    ));
    assert_eq!(
        service.poll(second).unwrap().phase(),
        Some(ZrRuntimeOperationPhase::Harvested)
    );
    assert_ne!(third, second);
}

#[test]
fn cancelled_operation_never_reaches_prepare_or_owner_apply() {
    let preparations = Arc::new(AtomicUsize::new(0));
    let applications = Arc::new(AtomicUsize::new(0));
    let mut service = RuntimeOperationService::new();
    service
        .register_handler(
            "test.echo",
            Arc::new(CountingHandler {
                preparations: preparations.clone(),
                applications: applications.clone(),
            }),
        )
        .unwrap();
    let handle = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.echo",
            serde_json::Value::Null,
        ))
        .unwrap();
    service.cancel(handle).expect("queued operation can cancel");

    let runtime = CoreRuntime::new();
    let mut world = World::empty();
    service.tick(&runtime.handle(), &mut world);
    let status = service.poll(handle).expect("cancel metadata is observable");
    assert_eq!(status.phase(), Some(ZrRuntimeOperationPhase::Cancelled));
    assert_eq!(
        status.detail_kind(),
        Some(ZrRuntimeOperationDetailKindV2::Cancelled)
    );
    assert_eq!(preparations.load(Ordering::SeqCst), 0);
    assert_eq!(applications.load(Ordering::SeqCst), 0);
    assert!(matches!(
        service.harvest(handle),
        Err(RuntimeOperationServiceError::OperationCancelled { .. })
    ));
}

#[test]
fn expired_operation_never_reaches_prepare_or_owner_apply() {
    let preparations = Arc::new(AtomicUsize::new(0));
    let applications = Arc::new(AtomicUsize::new(0));
    let mut service = RuntimeOperationService::new();
    service
        .register_handler(
            "test.echo",
            Arc::new(CountingHandler {
                preparations: preparations.clone(),
                applications: applications.clone(),
            }),
        )
        .unwrap();
    let handle = service
        .submit_with_deadline(
            ZrRuntimeOperationSubmitRequestV1::new(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                "test.echo",
                serde_json::Value::Null,
            ),
            Some(Instant::now()),
        )
        .unwrap();

    let runtime = CoreRuntime::new();
    let mut world = World::empty();
    service.tick(&runtime.handle(), &mut world);
    let status = service.poll(handle).expect("expiry metadata is observable");
    assert_eq!(status.phase(), Some(ZrRuntimeOperationPhase::Expired));
    assert_eq!(
        status.detail_kind(),
        Some(ZrRuntimeOperationDetailKindV2::DeadlineElapsed)
    );
    assert_eq!(preparations.load(Ordering::SeqCst), 0);
    assert_eq!(applications.load(Ordering::SeqCst), 0);
    assert!(matches!(
        service.harvest(handle),
        Err(RuntimeOperationServiceError::OperationExpired { .. })
    ));
}

#[test]
fn terminal_result_ttl_releases_payload_but_keeps_expired_metadata() {
    let mut service = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 2,
        max_in_flight_prepares: 1,
        max_retained_bytes: 1024,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::ZERO,
    });
    service
        .register_handler(
            "test.echo",
            Arc::new(CountingHandler {
                preparations: Arc::new(AtomicUsize::new(0)),
                applications: Arc::new(AtomicUsize::new(0)),
            }),
        )
        .unwrap();
    let handle = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.echo",
            serde_json::Value::Null,
        ))
        .unwrap();
    let runtime = CoreRuntime::new();
    let mut world = World::empty();
    tick_until_terminal(&service, &runtime, &mut world, handle);

    service.tick(&runtime.handle(), &mut world);
    let status = service.poll(handle).expect("TTL metadata is observable");
    assert_eq!(status.phase(), Some(ZrRuntimeOperationPhase::Expired));
    assert_eq!(
        status.detail_kind(),
        Some(ZrRuntimeOperationDetailKindV2::TerminalTtlElapsed)
    );
    assert!(matches!(
        service.harvest(handle),
        Err(RuntimeOperationServiceError::OperationExpired { .. })
    ));
}
