use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::Duration;

use zircon_runtime_interface::{
    ZrRuntimeOperationPhase, ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::*;

#[test]
fn operation_service_rejects_raw_admission_before_running_the_decoder() {
    let handler: Arc<dyn RuntimeOperationHandler> = Arc::new(CountingHandler {
        preparations: Arc::new(AtomicUsize::new(0)),
        applications: Arc::new(AtomicUsize::new(0)),
    });
    let mut service = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 1,
        max_in_flight_prepares: 1,
        max_retained_bytes: 8,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::from_secs(60),
    });
    service.register_handler("test.echo", handler).unwrap();

    let mut decoder_called = false;
    let result = service
        .submit_with_raw_admission(64, || {
            decoder_called = true;
            Ok::<_, ()>(ZrRuntimeOperationSubmitRequestV1::new(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                "test.echo",
                serde_json::Value::Null,
            ))
        })
        .expect("decoder errors are returned separately from admission errors");

    assert!(matches!(
        result,
        Err(RuntimeOperationServiceError::RetainedBytesCapacityReached { maximum: 8 })
    ));
    assert!(!decoder_called);
}

#[test]
fn operation_service_releases_raw_admission_when_external_decoder_fails() {
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
        service.submit_with_raw_admission(64, || Err::<ZrRuntimeOperationSubmitRequestV1, _>(())),
        Err(())
    ));

    let handle = service
        .submit_with_raw_admission(64, || {
            Ok::<_, ()>(ZrRuntimeOperationSubmitRequestV1::new(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                "test.echo",
                serde_json::Value::Null,
            ))
        })
        .expect("successful decoder result")
        .expect("released raw admission permits the next request");
    assert_eq!(
        service.poll(handle).unwrap().phase(),
        Some(ZrRuntimeOperationPhase::Queued)
    );
}
