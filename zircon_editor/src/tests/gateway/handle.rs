use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
};

use crate::core::gateway::{
    DetachedEditorRuntimeGateway, EditorRuntimeFrameDemand, EditorRuntimeGateway,
    EditorRuntimeGatewayHandle, GatewayError, RuntimeCapabilities, SessionProfileKind,
};

struct SessionOnlyGateway {
    session: u64,
    capabilities: Arc<RuntimeCapabilities>,
    tick_calls: Arc<AtomicUsize>,
}

impl SessionOnlyGateway {
    fn new(session: u64, tick_calls: Arc<AtomicUsize>) -> Self {
        Self {
            session,
            capabilities: Arc::new(RuntimeCapabilities::new(
                SessionProfileKind::Editor,
                ["editor.host.ui_shell"],
                [],
            )),
            tick_calls,
        }
    }
}

impl EditorRuntimeGateway for SessionOnlyGateway {
    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        self.capabilities.clone()
    }

    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::new(self.session)
    }

    fn tick_frame(&self) -> Result<EditorRuntimeFrameDemand, GatewayError> {
        self.tick_calls.fetch_add(1, Ordering::SeqCst);
        Ok(EditorRuntimeFrameDemand::Continuous)
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

struct BlockingGateway {
    capabilities: Arc<RuntimeCapabilities>,
    entered: Arc<Barrier>,
    release: Arc<Barrier>,
    drop_calls: Arc<AtomicUsize>,
}

impl EditorRuntimeGateway for BlockingGateway {
    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        self.capabilities.clone()
    }

    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::new(7)
    }

    fn tick_frame(&self) -> Result<EditorRuntimeFrameDemand, GatewayError> {
        self.entered.wait();
        self.release.wait();
        Ok(EditorRuntimeFrameDemand::OnDemand)
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit",
        })
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.poll",
        })
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.operation.harvest",
        })
    }
}

impl Drop for BlockingGateway {
    fn drop(&mut self) {
        self.drop_calls.fetch_add(1, Ordering::SeqCst);
    }
}

struct PanickingCapabilitiesGateway;

impl EditorRuntimeGateway for PanickingCapabilitiesGateway {
    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        panic!("capability snapshot construction failed")
    }

    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::new(99)
    }

    fn submit_operation(
        &self,
        _request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        unreachable!("panicking gateway must not be published")
    }

    fn poll_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        unreachable!("panicking gateway must not be published")
    }

    fn harvest_operation(
        &self,
        _handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        unreachable!("panicking gateway must not be published")
    }
}

#[test]
fn gateway_handle_replaces_detached_transport_without_changing_owner_identity() {
    let handle = EditorRuntimeGatewayHandle::detached();
    let clone = handle.clone();
    let tick_calls = Arc::new(AtomicUsize::new(0));
    assert_eq!(handle.session_handle(), ZrRuntimeSessionHandle::invalid());

    handle
        .replace(Arc::new(SessionOnlyGateway::new(41, tick_calls.clone())))
        .expect("replace detached runtime gateway");

    assert_eq!(handle.session_handle(), ZrRuntimeSessionHandle::new(41));
    assert_eq!(clone.session_handle(), ZrRuntimeSessionHandle::new(41));
    assert_eq!(
        clone.capabilities().session_profile(),
        SessionProfileKind::Editor
    );
    assert_eq!(
        clone.capabilities().core_capabilities(),
        &["editor.host.ui_shell"]
    );
    assert_eq!(
        clone
            .tick_frame()
            .expect("forward tick through stable handle"),
        EditorRuntimeFrameDemand::Continuous
    );
    assert_eq!(tick_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn gateway_handle_reuses_generation_bound_capability_snapshot() {
    let handle = EditorRuntimeGatewayHandle::new(Arc::new(SessionOnlyGateway::new(
        41,
        Arc::new(AtomicUsize::new(0)),
    )));

    let first = handle.capabilities();
    let second = handle.capabilities();

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(handle.generation(), 0);

    handle
        .replace(Arc::new(SessionOnlyGateway::new(
            42,
            Arc::new(AtomicUsize::new(0)),
        )))
        .expect("replace gateway capability generation");

    let replacement = handle.capabilities();
    assert!(!Arc::ptr_eq(&first, &replacement));
    assert_eq!(handle.generation(), 1);
}

#[test]
fn gateway_replacement_keeps_in_flight_generation_alive() {
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let drop_calls = Arc::new(AtomicUsize::new(0));
    let gateway = Arc::new(BlockingGateway {
        capabilities: Arc::new(RuntimeCapabilities::editor_default()),
        entered: entered.clone(),
        release: release.clone(),
        drop_calls: drop_calls.clone(),
    });
    let handle = EditorRuntimeGatewayHandle::new(gateway.clone());
    let worker_handle = handle.clone();
    let worker = std::thread::spawn(move || worker_handle.tick_frame());

    entered.wait();
    drop(gateway);
    let replacement_handle = handle.clone();
    let replacement = std::thread::spawn(move || {
        replacement_handle
            .replace(Arc::new(SessionOnlyGateway::new(
                9,
                Arc::new(AtomicUsize::new(0)),
            )))
            .expect("replace in-flight gateway generation");
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    while handle.generation() == 0 {
        assert!(
            Instant::now() < deadline,
            "replacement did not publish a new gateway generation"
        );
        std::thread::yield_now();
    }
    assert_eq!(drop_calls.load(Ordering::SeqCst), 0);

    release.wait();
    assert!(worker.join().expect("join in-flight gateway call").is_ok());
    replacement.join().expect("join gateway replacement");
    assert_eq!(drop_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn failed_gateway_replacement_preserves_generation_and_recovers_writer() {
    let handle = EditorRuntimeGatewayHandle::detached();

    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        handle.replace(Arc::new(PanickingCapabilitiesGateway));
    }));

    assert!(failure.is_err());
    assert_eq!(handle.generation(), 0);
    assert_eq!(handle.session_handle(), ZrRuntimeSessionHandle::invalid());

    handle
        .replace(Arc::new(SessionOnlyGateway::new(
            11,
            Arc::new(AtomicUsize::new(0)),
        )))
        .expect("recover writer after failed replacement");
    assert_eq!(handle.generation(), 1);
    assert_eq!(handle.session_handle(), ZrRuntimeSessionHandle::new(11));
}

#[test]
fn gateway_stable_path_has_no_shared_read_lock() {
    let source = include_str!("../../core/gateway/handle.rs");
    assert!(!source.contains("RwLock"));
    assert!(!source.contains("RwLockReadGuard"));
}

#[test]
fn detached_gateway_returns_typed_capability_error() {
    let detached = DetachedEditorRuntimeGateway;
    let error = detached
        .submit_operation(ZrRuntimeOperationSubmitRequestV1::new(
            zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1,
            "navigation.bake.scene",
            serde_json::Value::Null,
        ))
        .unwrap_err();

    assert!(matches!(
        error,
        GatewayError::CapabilityMissing {
            capability: "runtime.operation.submit"
        }
    ));
}
