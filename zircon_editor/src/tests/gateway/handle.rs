use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationProgressV1, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
};

use crate::core::gateway::{
    DetachedEditorRuntimeGateway, EditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError,
    RuntimeCapabilities, SessionProfileKind,
};

struct SessionOnlyGateway {
    session: u64,
    capabilities: RuntimeCapabilities,
    tick_calls: Arc<AtomicUsize>,
}

impl SessionOnlyGateway {
    fn new(session: u64, tick_calls: Arc<AtomicUsize>) -> Self {
        Self {
            session,
            capabilities: RuntimeCapabilities::new(
                SessionProfileKind::Editor,
                ["editor.host.ui_shell"],
                [],
            ),
            tick_calls,
        }
    }
}

impl EditorRuntimeGateway for SessionOnlyGateway {
    fn capabilities(&self) -> RuntimeCapabilities {
        self.capabilities.clone()
    }

    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::new(self.session)
    }

    fn tick_frame(&self) -> Result<bool, GatewayError> {
        self.tick_calls.fetch_add(1, Ordering::SeqCst);
        Ok(true)
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
    ) -> Result<ZrRuntimeOperationProgressV1, GatewayError> {
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

#[test]
fn gateway_handle_replaces_detached_transport_without_changing_owner_identity() {
    let handle = EditorRuntimeGatewayHandle::detached();
    let clone = handle.clone();
    let tick_calls = Arc::new(AtomicUsize::new(0));
    assert_eq!(handle.session_handle(), ZrRuntimeSessionHandle::invalid());

    handle.replace(Arc::new(SessionOnlyGateway::new(41, tick_calls.clone())));

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
    assert!(clone
        .tick_frame()
        .expect("forward tick through stable handle"));
    assert_eq!(tick_calls.load(Ordering::SeqCst), 1);
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
