use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1,
};

use super::{EditorRuntimeGatewayHandle, GatewayError, GatewayOrigin, GatewaySessionIdentity};

/// Identity-pinned runtime capability used by one editor command execution chain.
///
/// Unlike the replaceable gateway handle, this route retains one immutable gateway generation so
/// submit, poll, and harvest cannot be redirected to different runtime sessions mid-command.
#[derive(Clone)]
pub struct EditorRuntimeOperationRoute {
    origin: GatewayOrigin,
}

impl EditorRuntimeOperationRoute {
    pub fn capture_at_identity(
        gateway: &EditorRuntimeGatewayHandle,
        expected_identity: &GatewaySessionIdentity,
    ) -> Result<Self, GatewayError> {
        let lease = gateway.current_lease();
        if lease.identity() != expected_identity {
            return Err(GatewayError::StaleGeneration {
                expected_generation: expected_identity.gateway_generation(),
                current_generation: lease.generation(),
            });
        }
        Ok(Self {
            origin: lease.origin(),
        })
    }

    pub fn identity(&self) -> &GatewaySessionIdentity {
        self.origin.identity()
    }

    pub fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        self.origin.gateway().submit_operation(request)
    }

    pub fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        self.origin.gateway().poll_operation(handle)
    }

    pub fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        self.origin.gateway().harvest_operation(handle)
    }
}
