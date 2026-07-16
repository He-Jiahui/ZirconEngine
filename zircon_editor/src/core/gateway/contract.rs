use zircon_runtime_interface::{
    ProfileControlRequest, ProfileControlResponse, ZrRuntimeEventV1, ZrRuntimeFrameV1,
    ZrRuntimeOperationHandle, ZrRuntimeOperationProgressV1, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimePluginEventDeliveryV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1,
};

use super::GatewayError;

pub trait EditorRuntimeGateway: Send + Sync {
    fn session_handle(&self) -> ZrRuntimeSessionHandle;

    fn tick_frame(&self) -> Result<bool, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.frame.tick",
        })
    }

    fn handle_event(&self, _event: ZrRuntimeEventV1) -> Result<(), GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.event.handle",
        })
    }

    fn capture_frame(
        &self,
        _viewport: ZrRuntimeViewportHandle,
        _size: ZrRuntimeViewportSizeV1,
    ) -> Result<ZrRuntimeFrameV1, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.frame.capture",
        })
    }

    fn profile_control(
        &self,
        _request: &ProfileControlRequest,
    ) -> Result<Option<ProfileControlResponse>, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.profile.control",
        })
    }

    fn subscribe_plugin_event(
        &self,
        _event_id: &str,
        _payload_schema: &str,
    ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.plugin_event.subscribe",
        })
    }

    fn unsubscribe_plugin_event(
        &self,
        _subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.plugin_event.unsubscribe",
        })
    }

    fn drain_plugin_events(
        &self,
        _subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<Vec<ZrRuntimePluginEventDeliveryV1>, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.plugin_event.drain",
        })
    }

    fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError>;

    fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationProgressV1, GatewayError>;

    fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError>;
}
