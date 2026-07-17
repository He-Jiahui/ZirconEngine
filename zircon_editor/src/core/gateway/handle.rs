use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use zircon_runtime::scene::World;
use zircon_runtime_interface::{
    ProfileControlRequest, ProfileControlResponse, ZrRuntimeEventV1, ZrRuntimeOperationHandle,
    ZrRuntimeOperationProgressV1, ZrRuntimeOperationResultV1, ZrRuntimeOperationSubmitRequestV1,
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1,
};

use super::{
    DetachedEditorRuntimeGateway, EditorRuntimeFrame, EditorRuntimeGateway, GatewayError,
    RuntimeCapabilities, SharedEditorRuntimeGateway,
};

/// Stable editor service identity whose transport can be attached after module startup.
#[derive(Clone)]
pub struct EditorRuntimeGatewayHandle {
    inner: Arc<RwLock<SharedEditorRuntimeGateway>>,
}

impl EditorRuntimeGatewayHandle {
    pub fn new(gateway: SharedEditorRuntimeGateway) -> Self {
        Self {
            inner: Arc::new(RwLock::new(gateway)),
        }
    }

    pub fn detached() -> Self {
        Self::new(Arc::new(DetachedEditorRuntimeGateway))
    }

    pub fn replace(&self, gateway: SharedEditorRuntimeGateway) {
        *self.write() = gateway;
    }

    fn snapshot(&self) -> SharedEditorRuntimeGateway {
        self.read().clone()
    }

    fn read(&self) -> RwLockReadGuard<'_, SharedEditorRuntimeGateway> {
        self.inner
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write(&self) -> RwLockWriteGuard<'_, SharedEditorRuntimeGateway> {
        self.inner
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl EditorRuntimeGatewayHandle {
    pub fn capabilities(&self) -> RuntimeCapabilities {
        self.snapshot().capabilities()
    }

    pub fn session_handle(&self) -> ZrRuntimeSessionHandle {
        self.snapshot().session_handle()
    }

    pub fn with_world(&self, read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
        self.snapshot().with_world(read)
    }

    pub fn with_world_mut(&self, write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        self.snapshot().with_world_mut(write)
    }

    pub fn tick_frame(&self) -> Result<bool, GatewayError> {
        self.snapshot().tick_frame()
    }

    pub fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), GatewayError> {
        self.snapshot().handle_event(event)
    }

    pub fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<EditorRuntimeFrame, GatewayError> {
        self.snapshot().capture_frame(viewport, size)
    }

    pub fn profile_control(
        &self,
        request: &ProfileControlRequest,
    ) -> Result<Option<ProfileControlResponse>, GatewayError> {
        self.snapshot().profile_control(request)
    }

    pub fn subscribe_plugin_event(
        &self,
        event_id: &str,
        payload_schema: &str,
    ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, GatewayError> {
        self.snapshot()
            .subscribe_plugin_event(event_id, payload_schema)
    }

    pub fn unsubscribe_plugin_event(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, GatewayError> {
        self.snapshot().unsubscribe_plugin_event(subscription)
    }

    pub fn drain_plugin_events(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<Vec<ZrRuntimePluginEventDeliveryV1>, GatewayError> {
        self.snapshot().drain_plugin_events(subscription)
    }

    pub fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        self.snapshot().submit_operation(request)
    }

    pub fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationProgressV1, GatewayError> {
        self.snapshot().poll_operation(handle)
    }

    pub fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        self.snapshot().harvest_operation(handle)
    }
}

impl EditorRuntimeGateway for EditorRuntimeGatewayHandle {
    fn capabilities(&self) -> RuntimeCapabilities {
        EditorRuntimeGatewayHandle::capabilities(self)
    }

    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        EditorRuntimeGatewayHandle::session_handle(self)
    }

    fn with_world(&self, read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
        EditorRuntimeGatewayHandle::with_world(self, read)
    }

    fn with_world_mut(&self, write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        EditorRuntimeGatewayHandle::with_world_mut(self, write)
    }

    fn tick_frame(&self) -> Result<bool, GatewayError> {
        EditorRuntimeGatewayHandle::tick_frame(self)
    }

    fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), GatewayError> {
        EditorRuntimeGatewayHandle::handle_event(self, event)
    }

    fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<EditorRuntimeFrame, GatewayError> {
        EditorRuntimeGatewayHandle::capture_frame(self, viewport, size)
    }

    fn profile_control(
        &self,
        request: &ProfileControlRequest,
    ) -> Result<Option<ProfileControlResponse>, GatewayError> {
        EditorRuntimeGatewayHandle::profile_control(self, request)
    }

    fn subscribe_plugin_event(
        &self,
        event_id: &str,
        payload_schema: &str,
    ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, GatewayError> {
        EditorRuntimeGatewayHandle::subscribe_plugin_event(self, event_id, payload_schema)
    }

    fn unsubscribe_plugin_event(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, GatewayError> {
        EditorRuntimeGatewayHandle::unsubscribe_plugin_event(self, subscription)
    }

    fn drain_plugin_events(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<Vec<ZrRuntimePluginEventDeliveryV1>, GatewayError> {
        EditorRuntimeGatewayHandle::drain_plugin_events(self, subscription)
    }

    fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        EditorRuntimeGatewayHandle::submit_operation(self, request)
    }

    fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationProgressV1, GatewayError> {
        EditorRuntimeGatewayHandle::poll_operation(self, handle)
    }

    fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        EditorRuntimeGatewayHandle::harvest_operation(self, handle)
    }
}
