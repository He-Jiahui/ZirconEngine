use zircon_runtime::scene::World;
use zircon_runtime_interface::{
    ProfileControlRequest, ProfileControlResponse, ZrRuntimeEventV1, ZrRuntimeOperationHandle,
    ZrRuntimeOperationProgressV1, ZrRuntimeOperationResultV1, ZrRuntimeOperationSubmitRequestV1,
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1,
};

use super::{GatewayError, RuntimeCapabilities};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorRuntimeFrame {
    abi_version: u32,
    width: u32,
    height: u32,
    generation: u64,
    rgba: Vec<u8>,
}

impl EditorRuntimeFrame {
    pub fn new(abi_version: u32, width: u32, height: u32, generation: u64, rgba: Vec<u8>) -> Self {
        Self {
            abi_version,
            width,
            height,
            generation,
            rgba,
        }
    }

    pub fn empty(abi_version: u32) -> Self {
        Self::new(abi_version, 0, 0, 0, Vec::new())
    }

    pub fn abi_version(&self) -> u32 {
        self.abi_version
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }
}

pub trait EditorRuntimeGateway: Send + Sync {
    fn capabilities(&self) -> RuntimeCapabilities {
        RuntimeCapabilities::unavailable().clone()
    }

    fn session_handle(&self) -> ZrRuntimeSessionHandle;

    fn with_world(&self, _read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
        Err(GatewayError::RequiresSerializedAccess)
    }

    fn with_world_mut(&self, _write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        Err(GatewayError::RequiresSerializedAccess)
    }

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
    ) -> Result<EditorRuntimeFrame, GatewayError> {
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
