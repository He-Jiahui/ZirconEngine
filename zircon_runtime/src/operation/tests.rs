use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde_json::json;
use zircon_runtime_interface::{
    ZrRuntimeOperationPhase, ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use crate::core::runtime::CoreRuntime;
use crate::scene::World;

use super::service::RuntimeOperationLimits;
use super::{
    RuntimeOperationContext, RuntimeOperationHandler, RuntimeOperationHandlerError,
    RuntimeOperationService, RuntimeOperationServiceError,
};

struct CountingHandler {
    preparations: Arc<AtomicUsize>,
    applications: Arc<AtomicUsize>,
}

struct PanicPrepareHandler;

impl RuntimeOperationHandler for PanicPrepareHandler {
    fn prepare(
        &self,
        _payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        panic!("prepare panic used to verify terminal containment")
    }

    fn apply(
        &self,
        _context: RuntimeOperationContext<'_>,
        _payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        panic!("failed preparation must never reach owner apply")
    }
}

impl RuntimeOperationHandler for CountingHandler {
    fn prepare(
        &self,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        self.preparations.fetch_add(1, Ordering::SeqCst);
        Ok(payload)
    }

    fn apply(
        &self,
        _context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        self.applications.fetch_add(1, Ordering::SeqCst);
        Ok(json!({"echo": payload}))
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
    assert_eq!(queued.phase, ZrRuntimeOperationPhase::Queued);
    assert_eq!(preparations.load(Ordering::SeqCst), 0);
    assert_eq!(applications.load(Ordering::SeqCst), 0);

    let completed = tick_until_terminal(&service, &runtime, &mut world, handle);
    assert_eq!(completed.phase, ZrRuntimeOperationPhase::Completed);
    assert_eq!(preparations.load(Ordering::SeqCst), 1);
    assert_eq!(applications.load(Ordering::SeqCst), 1);

    let result = service.harvest(handle).unwrap();
    assert_eq!(result.succeeded_output().unwrap()["echo"]["value"], 9);
    assert!(matches!(
        service.harvest(handle),
        Err(RuntimeOperationServiceError::UnknownHandle { .. })
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
    assert_eq!(terminal.phase, ZrRuntimeOperationPhase::Failed);
    assert!(service
        .harvest(handle)
        .unwrap()
        .failure()
        .is_some_and(|error| error.contains("prepare panicked")));
}

fn tick_until_terminal(
    service: &RuntimeOperationService,
    runtime: &CoreRuntime,
    world: &mut World,
    handle: zircon_runtime_interface::ZrRuntimeOperationHandle,
) -> zircon_runtime_interface::ZrRuntimeOperationProgressV1 {
    for _ in 0..1024 {
        service.tick(&runtime.handle(), world);
        let progress = service.poll(handle).unwrap();
        if progress.phase.is_terminal() {
            return progress;
        }
        std::thread::yield_now();
    }
    panic!("bounded operation task did not reach a terminal phase");
}

#[test]
fn terminal_harvest_uses_one_task_table_entry_lookup() {
    let source = include_str!("service.rs");
    let start = source
        .find("    pub fn harvest(")
        .expect("operation harvest owner");
    let end = source[start..]
        .find("    fn dispatch_queued_prepares(")
        .map(|offset| start + offset)
        .expect("operation harvest owner end");
    let harvest_source = &source[start..end];

    assert!(
        harvest_source.contains("state.tasks.entry(handle)"),
        "harvest must validate and remove through one occupied-entry lookup"
    );
    assert!(
        !harvest_source.contains("state.tasks.get(&handle)"),
        "harvest must not probe the task table before removing the same handle"
    );
}
