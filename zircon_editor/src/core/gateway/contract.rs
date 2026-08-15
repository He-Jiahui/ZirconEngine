use std::sync::Arc;
use std::time::Duration;

use zircon_runtime::scene::World;
use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchRegistration, WatchToken, WorldQuery, WorldQueryResult,
};
use zircon_runtime_interface::{
    ProfileControlRequest, ProfileControlResponse, ZrRuntimeBindViewportSurfaceRequestV1,
    ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeOperationHandle,
    ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1,
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1,
};

use super::{EditorRuntimeHighlightSet, GatewayError, RuntimeCapabilities};

pub(crate) trait EditorRuntimeFramePixels {
    fn rgba(&self) -> &[u8];

    fn release(self: Box<Self>) -> Result<(), GatewayError>;
}

#[derive(Debug)]
struct EditorOwnedRuntimeFramePixels {
    rgba: Vec<u8>,
}

impl EditorRuntimeFramePixels for EditorOwnedRuntimeFramePixels {
    fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    fn release(self: Box<Self>) -> Result<(), GatewayError> {
        Ok(())
    }
}

pub struct EditorRuntimeFrame {
    abi_version: u32,
    width: u32,
    height: u32,
    generation: u64,
    pixels: Box<dyn EditorRuntimeFramePixels>,
}

impl std::fmt::Debug for EditorRuntimeFrame {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EditorRuntimeFrame")
            .field("abi_version", &self.abi_version)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("generation", &self.generation)
            .field("rgba_len", &self.rgba().len())
            .finish()
    }
}

impl EditorRuntimeFrame {
    pub fn new(abi_version: u32, width: u32, height: u32, generation: u64, rgba: Vec<u8>) -> Self {
        Self::from_pixels(
            abi_version,
            width,
            height,
            generation,
            Box::new(EditorOwnedRuntimeFramePixels { rgba }),
        )
    }

    pub(crate) fn from_pixels(
        abi_version: u32,
        width: u32,
        height: u32,
        generation: u64,
        pixels: Box<dyn EditorRuntimeFramePixels>,
    ) -> Self {
        Self {
            abi_version,
            width,
            height,
            generation,
            pixels,
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
        self.pixels.rgba()
    }

    pub fn release(self) -> Result<(), GatewayError> {
        self.pixels.release()
    }
}

/// Editor-facing cadence requested by the runtime after a successful frame tick.
///
/// The serialized gateway converts the runtime ABI into this contract once, so host and UI
/// scheduling never need to interpret raw ABI kinds or delay fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorRuntimeFrameDemand {
    OnDemand,
    SleepUntil(Duration),
    Continuous,
}

/// One bounded plugin-event page returned by the editor runtime gateway.
///
/// `runtime_drain_elapsed` covers the runtime ABI call, including the runtime's page encoding;
/// `decode_elapsed` covers editor-owned decoding and buffer release after the call succeeds.
#[derive(Debug)]
pub struct EditorRuntimePluginEventPage {
    deliveries: Vec<ZrRuntimePluginEventDeliveryV1>,
    encoded_bytes: usize,
    runtime_drain_elapsed: Duration,
    decode_elapsed: Duration,
    runtime_remaining_deliveries: usize,
    runtime_oldest_pending_age_millis: u64,
}

impl EditorRuntimePluginEventPage {
    pub fn new(
        deliveries: Vec<ZrRuntimePluginEventDeliveryV1>,
        encoded_bytes: usize,
        runtime_drain_elapsed: Duration,
        decode_elapsed: Duration,
    ) -> Self {
        Self {
            deliveries,
            encoded_bytes,
            runtime_drain_elapsed,
            decode_elapsed,
            runtime_remaining_deliveries: 0,
            runtime_oldest_pending_age_millis: 0,
        }
    }

    pub fn with_runtime_backlog(
        mut self,
        runtime_remaining_deliveries: usize,
        runtime_oldest_pending_age_millis: u64,
    ) -> Self {
        self.runtime_remaining_deliveries = runtime_remaining_deliveries;
        self.runtime_oldest_pending_age_millis = runtime_oldest_pending_age_millis;
        self
    }

    pub(crate) fn synthetic(deliveries: Vec<ZrRuntimePluginEventDeliveryV1>) -> Self {
        Self::new(deliveries, 0, Duration::ZERO, Duration::ZERO)
    }

    pub fn deliveries(&self) -> &[ZrRuntimePluginEventDeliveryV1] {
        &self.deliveries
    }

    pub fn into_deliveries(self) -> Vec<ZrRuntimePluginEventDeliveryV1> {
        self.deliveries
    }

    pub fn is_empty(&self) -> bool {
        self.deliveries.is_empty()
    }

    pub fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }

    pub fn runtime_drain_elapsed(&self) -> Duration {
        self.runtime_drain_elapsed
    }

    pub fn decode_elapsed(&self) -> Duration {
        self.decode_elapsed
    }

    pub fn runtime_remaining_deliveries(&self) -> usize {
        self.runtime_remaining_deliveries
    }

    pub fn runtime_oldest_pending_age_millis(&self) -> u64 {
        self.runtime_oldest_pending_age_millis
    }
}

pub trait EditorRuntimeGateway: Send + Sync {
    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        RuntimeCapabilities::unavailable()
    }

    fn session_handle(&self) -> ZrRuntimeSessionHandle;

    fn with_world(&self, _read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
        Err(GatewayError::RequiresSerializedAccess)
    }

    fn with_world_mut(&self, _write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        Err(GatewayError::RequiresSerializedAccess)
    }

    fn query_world(&self, _query: WorldQuery) -> Result<WorldQueryResult, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.world_sync.query",
        })
    }

    fn watch_world(&self, _registration: WatchRegistration) -> Result<WatchToken, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.world_sync.watch",
        })
    }

    fn unwatch_world(&self, _token: WatchToken) -> Result<bool, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.world_sync.unwatch",
        })
    }

    fn drain_world_invalidations(&self) -> Result<Vec<InvalidationBatch>, GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.world_sync.drain",
        })
    }

    fn tick_frame(&self) -> Result<EditorRuntimeFrameDemand, GatewayError> {
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

    fn bind_viewport_surface(
        &self,
        _request: ZrRuntimeBindViewportSurfaceRequestV1,
    ) -> Result<(), GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.viewport.surface.bind",
        })
    }

    fn unbind_viewport_surface(
        &self,
        _viewport: ZrRuntimeViewportHandle,
    ) -> Result<(), GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.viewport.surface.unbind",
        })
    }

    fn present_viewport(&self, _request: ZrRuntimeFrameRequestV1) -> Result<(), GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.viewport.present",
        })
    }

    fn submit_highlight_set(&self, _set: EditorRuntimeHighlightSet) -> Result<(), GatewayError> {
        Err(GatewayError::CapabilityMissing {
            capability: "runtime.editor_overlay.highlight_set",
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
    ) -> Result<EditorRuntimePluginEventPage, GatewayError> {
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
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError>;

    fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError>;
}
