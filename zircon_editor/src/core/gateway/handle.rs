use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use zircon_runtime_interface::{
    ProfileControlRequest, ProfileControlResponse, ZrRuntimeEventV1, ZrRuntimeFrameV1,
    ZrRuntimeOperationHandle, ZrRuntimeOperationProgressV1, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimePluginEventDeliveryV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1,
};

use super::{
    DetachedEditorRuntimeGateway, EditorRuntimeGateway, GatewayError, SharedEditorRuntimeGateway,
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

impl EditorRuntimeGateway for EditorRuntimeGatewayHandle {
    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        self.snapshot().session_handle()
    }

    fn tick_frame(&self) -> Result<bool, GatewayError> {
        self.snapshot().tick_frame()
    }

    fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), GatewayError> {
        self.snapshot().handle_event(event)
    }

    fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<ZrRuntimeFrameV1, GatewayError> {
        self.snapshot().capture_frame(viewport, size)
    }

    fn profile_control(
        &self,
        request: &ProfileControlRequest,
    ) -> Result<Option<ProfileControlResponse>, GatewayError> {
        self.snapshot().profile_control(request)
    }

    fn subscribe_plugin_event(
        &self,
        event_id: &str,
        payload_schema: &str,
    ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, GatewayError> {
        self.snapshot()
            .subscribe_plugin_event(event_id, payload_schema)
    }

    fn unsubscribe_plugin_event(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, GatewayError> {
        self.snapshot().unsubscribe_plugin_event(subscription)
    }

    fn drain_plugin_events(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<Vec<ZrRuntimePluginEventDeliveryV1>, GatewayError> {
        self.snapshot().drain_plugin_events(subscription)
    }

    fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        self.snapshot().submit_operation(request)
    }

    fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationProgressV1, GatewayError> {
        self.snapshot().poll_operation(handle)
    }

    fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        self.snapshot().harvest_operation(handle)
    }
}
