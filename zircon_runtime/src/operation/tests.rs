use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use serde_json::json;
use zircon_runtime_interface::{
    ZrRuntimeOperationPhase, ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use crate::core::runtime::CoreRuntime;
use crate::scene::World;

use super::{
    RuntimeOperationContext, RuntimeOperationHandler, RuntimeOperationHandlerError,
    RuntimeOperationService, RuntimeOperationServiceError,
};

struct CountingHandler {
    executions: Arc<AtomicUsize>,
}

impl RuntimeOperationHandler for CountingHandler {
    fn execute(
        &self,
        _context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        self.executions.fetch_add(1, Ordering::SeqCst);
        Ok(json!({"echo": payload}))
    }
}

#[test]
fn operation_service_runs_only_after_running_progress_is_observable() {
    let executions = Arc::new(AtomicUsize::new(0));
    let mut service = RuntimeOperationService::new();
    service
        .register_handler(
            "test.echo",
            Arc::new(CountingHandler {
                executions: executions.clone(),
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

    let running = service
        .poll(
            RuntimeOperationContext::new(&runtime.handle(), &mut world),
            handle,
        )
        .unwrap();
    assert_eq!(running.phase, ZrRuntimeOperationPhase::Running);
    assert_eq!(executions.load(Ordering::SeqCst), 0);

    let completed = service
        .poll(
            RuntimeOperationContext::new(&runtime.handle(), &mut world),
            handle,
        )
        .unwrap();
    assert_eq!(completed.phase, ZrRuntimeOperationPhase::Completed);
    assert_eq!(executions.load(Ordering::SeqCst), 1);

    let result = service.harvest(handle).unwrap();
    assert_eq!(result.succeeded_output().unwrap()["echo"]["value"], 9);
    assert!(matches!(
        service.harvest(handle),
        Err(RuntimeOperationServiceError::UnknownHandle { .. })
    ));
}

#[test]
fn operation_service_rejects_unknown_operations_duplicate_handlers_and_early_harvest() {
    let executions = Arc::new(AtomicUsize::new(0));
    let handler: Arc<dyn RuntimeOperationHandler> = Arc::new(CountingHandler { executions });
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
