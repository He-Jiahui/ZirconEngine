use std::sync::Arc;

use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchRegistration, WatchToken, WorldQuery, WorldQueryResult,
};
use zircon_runtime_interface::{
    ProfileControlRequest, ProfileControlResponse, ZrRuntimeBindViewportSurfaceRequestV1,
    ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeOperationHandle,
    ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1,
};

use super::super::{
    EditorRuntimeFrame, EditorRuntimeFrameDemand, EditorRuntimeGateway, EditorRuntimeHighlightSet,
    EditorRuntimePluginEventPage, GatewayError, RuntimeCapabilities,
};
use super::gateway::SessionGateway;

impl EditorRuntimeGateway for SessionGateway {
    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        self.capabilities.clone()
    }

    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        self.session
    }

    fn query_world(&self, query: WorldQuery) -> Result<WorldQueryResult, GatewayError> {
        SessionGateway::query_world(self, query)
    }

    fn watch_world(&self, registration: WatchRegistration) -> Result<WatchToken, GatewayError> {
        SessionGateway::watch_world(self, registration)
    }

    fn unwatch_world(&self, token: WatchToken) -> Result<bool, GatewayError> {
        SessionGateway::unwatch_world(self, token)
    }

    fn drain_world_invalidations(&self) -> Result<Vec<InvalidationBatch>, GatewayError> {
        SessionGateway::drain_world_invalidations(self)
    }

    fn tick_frame(&self) -> Result<EditorRuntimeFrameDemand, GatewayError> {
        SessionGateway::tick_frame(self)
    }

    fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), GatewayError> {
        SessionGateway::handle_event(self, event)
    }

    fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<EditorRuntimeFrame, GatewayError> {
        SessionGateway::capture_frame(self, viewport, size)
    }

    fn bind_viewport_surface(
        &self,
        request: ZrRuntimeBindViewportSurfaceRequestV1,
    ) -> Result<(), GatewayError> {
        SessionGateway::bind_viewport_surface(self, request)
    }

    fn unbind_viewport_surface(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<(), GatewayError> {
        SessionGateway::unbind_viewport_surface(self, viewport)
    }

    fn present_viewport(&self, request: ZrRuntimeFrameRequestV1) -> Result<(), GatewayError> {
        SessionGateway::present_viewport(self, request)
    }

    fn submit_highlight_set(&self, set: EditorRuntimeHighlightSet) -> Result<(), GatewayError> {
        SessionGateway::submit_highlight_set(self, set)
    }

    fn profile_control(
        &self,
        request: &ProfileControlRequest,
    ) -> Result<Option<ProfileControlResponse>, GatewayError> {
        SessionGateway::profile_control(self, request)
    }

    fn subscribe_plugin_event(
        &self,
        event_id: &str,
        payload_schema: &str,
    ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, GatewayError> {
        SessionGateway::subscribe_plugin_event(self, event_id, payload_schema)
    }

    fn unsubscribe_plugin_event(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, GatewayError> {
        SessionGateway::unsubscribe_plugin_event(self, subscription)
    }

    fn drain_plugin_events(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<EditorRuntimePluginEventPage, GatewayError> {
        SessionGateway::drain_plugin_events(self, subscription)
    }

    fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        SessionGateway::submit_operation(self, request)
    }

    fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        SessionGateway::poll_operation(self, handle)
    }

    fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        SessionGateway::harvest_operation(self, handle)
    }
}
