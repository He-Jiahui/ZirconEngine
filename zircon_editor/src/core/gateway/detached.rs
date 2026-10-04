use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
};

use super::{EditorRuntimeGateway, GatewayError, GatewaySessionIdentity};

#[derive(Debug, Default)]
/// 无会话时的稳定端点；宿主仍可保留统一接口，但运行时操作必须显式失败。
pub struct DetachedEditorRuntimeGateway;

impl EditorRuntimeGateway for DetachedEditorRuntimeGateway {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::invalid()
    }

    fn session_identity(&self) -> GatewaySessionIdentity {
        GatewaySessionIdentity::detached()
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
