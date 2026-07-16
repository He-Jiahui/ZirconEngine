use std::sync::Arc;

use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationProgressV1, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
};

use crate::core::gateway::{
    DetachedEditorRuntimeGateway, EditorRuntimeGateway, EditorRuntimeGatewayHandle, GatewayError,
};

struct SessionOnlyGateway(u64);

impl EditorRuntimeGateway for SessionOnlyGateway {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::new(self.0)
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
    assert_eq!(handle.session_handle(), ZrRuntimeSessionHandle::invalid());

    handle.replace(Arc::new(SessionOnlyGateway(41)));

    assert_eq!(handle.session_handle(), ZrRuntimeSessionHandle::new(41));
    assert_eq!(clone.session_handle(), ZrRuntimeSessionHandle::new(41));
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
