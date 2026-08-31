use std::sync::{Arc, Mutex, MutexGuard};

use arc_swap::ArcSwap;

use zircon_runtime::scene::World;
use zircon_runtime_interface::runtime_build_set::ZrRuntimeModuleCompositionReceiptV1;
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

use super::{
    DetachedEditorRuntimeGateway, EditorRuntimeFrame, EditorRuntimeFrameDemand,
    EditorRuntimeGateway, EditorRuntimeHighlightSet, EditorRuntimePluginEventPage, GatewayError,
    GatewaySessionIdentity, RuntimeCapabilities, SharedEditorRuntimeGateway,
};

/// Stable editor service identity whose transport can be attached after module startup.
#[derive(Clone)]
pub struct EditorRuntimeGatewayHandle {
    inner: Arc<GatewayOwner>,
}

struct GatewayOwner {
    current: ArcSwap<GatewayGeneration>,
    replacement: Mutex<()>,
}

struct GatewayGeneration {
    id: u64,
    identity: GatewaySessionIdentity,
    gateway: SharedEditorRuntimeGateway,
    capabilities: Arc<RuntimeCapabilities>,
    module_composition_receipt: Option<Arc<ZrRuntimeModuleCompositionReceiptV1>>,
}

/// Immutable origin endpoint retained by a resource allocated against one transport generation.
///
/// This owns no replacement lock. Long-lived editor resources keep an origin solely to route their
/// opaque cleanup operations back to the endpoint that created them.
#[derive(Clone)]
pub(crate) struct GatewayOrigin {
    generation: Arc<GatewayGeneration>,
}

impl GatewayOrigin {
    pub(crate) fn generation(&self) -> u64 {
        self.generation.id
    }

    pub(crate) fn session_handle(&self) -> ZrRuntimeSessionHandle {
        self.generation.identity.runtime_session()
    }

    pub(crate) fn identity(&self) -> &GatewaySessionIdentity {
        &self.generation.identity
    }

    pub(crate) fn gateway(&self) -> &SharedEditorRuntimeGateway {
        &self.generation.gateway
    }
}

/// A short-lived immutable view of one runtime transport generation.
///
/// A lease clones its origin's `Arc` and never holds the replacement mutex. It is intentionally
/// used for one bounded operation chain; long-lived state must retain [`GatewayOrigin`] instead.
#[derive(Clone)]
pub(crate) struct GatewayLease {
    origin: GatewayOrigin,
}

impl GatewayLease {
    pub(crate) fn origin(&self) -> GatewayOrigin {
        self.origin.clone()
    }

    pub(crate) fn generation(&self) -> u64 {
        self.origin.generation()
    }

    pub(crate) fn session_handle(&self) -> ZrRuntimeSessionHandle {
        self.origin.session_handle()
    }

    pub(crate) fn identity(&self) -> &GatewaySessionIdentity {
        self.origin.identity()
    }

    pub(crate) fn gateway(&self) -> &SharedEditorRuntimeGateway {
        self.origin.gateway()
    }

    pub(crate) fn tick_frame(&self) -> Result<EditorRuntimeFrameDemand, GatewayError> {
        self.gateway().tick_frame()
    }
}

impl GatewayGeneration {
    fn new(id: u64, gateway: SharedEditorRuntimeGateway, play_instance: Option<u64>) -> Self {
        let capabilities = gateway.capabilities();
        let module_composition_receipt = gateway.module_composition_receipt();
        Self {
            id,
            identity: gateway
                .session_identity()
                .with_gateway_generation(id)
                .with_play_instance(play_instance),
            gateway,
            capabilities,
            module_composition_receipt,
        }
    }
}

impl EditorRuntimeGatewayHandle {
    pub fn new(gateway: SharedEditorRuntimeGateway) -> Self {
        Self {
            inner: Arc::new(GatewayOwner {
                current: ArcSwap::from_pointee(GatewayGeneration::new(0, gateway, None)),
                replacement: Mutex::new(()),
            }),
        }
    }

    pub fn detached() -> Self {
        Self::new(Arc::new(DetachedEditorRuntimeGateway))
    }

    pub fn replace(&self, gateway: SharedEditorRuntimeGateway) -> Result<(), GatewayError> {
        self.replace_for_play(gateway, None)
    }

    pub(crate) fn replace_for_play(
        &self,
        gateway: SharedEditorRuntimeGateway,
        play_instance: Option<u64>,
    ) -> Result<(), GatewayError> {
        let _replacement = self.replacement_lock();
        let next_generation = next_generation(self.current_lease().generation())?;
        self.inner.current.store(Arc::new(GatewayGeneration::new(
            next_generation,
            gateway,
            play_instance,
        )));
        Ok(())
    }

    /// Detaches only the generation observed by a lifecycle owner. The identity check and
    /// detached-generation publication share the replacement lock, so a cloned handle cannot
    /// replace the gateway between those two operations.
    pub(crate) fn detach_at_identity(
        &self,
        expected_identity: &GatewaySessionIdentity,
    ) -> Result<(), GatewayError> {
        let _replacement = self.replacement_lock();
        let current = self.current_lease();
        if current.identity() != expected_identity {
            return Err(GatewayError::StaleGeneration {
                expected_generation: expected_identity.gateway_generation(),
                current_generation: current.generation(),
            });
        }
        let next_generation = next_generation(current.generation())?;
        self.inner.current.store(Arc::new(GatewayGeneration::new(
            next_generation,
            Arc::new(DetachedEditorRuntimeGateway),
            None,
        )));
        Ok(())
    }

    pub fn generation(&self) -> u64 {
        self.current_lease().generation()
    }

    pub fn identity(&self) -> GatewaySessionIdentity {
        self.current_lease().identity().clone()
    }

    pub fn module_composition_receipt(&self) -> Option<Arc<ZrRuntimeModuleCompositionReceiptV1>> {
        self.current_lease()
            .generation
            .module_composition_receipt
            .clone()
    }

    pub(crate) fn current_lease(&self) -> GatewayLease {
        GatewayLease {
            origin: GatewayOrigin {
                generation: self.inner.current.load_full(),
            },
        }
    }

    fn replacement_lock(&self) -> MutexGuard<'_, ()> {
        self.inner
            .replacement
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn next_generation(current: u64) -> Result<u64, GatewayError> {
    current
        .checked_add(1)
        .ok_or(GatewayError::GenerationExhausted)
}

impl EditorRuntimeGatewayHandle {
    pub fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        self.current_lease().generation.capabilities.clone()
    }

    pub fn session_handle(&self) -> ZrRuntimeSessionHandle {
        self.current_lease().session_handle()
    }

    pub fn with_world(&self, read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
        self.current_lease().gateway().with_world(read)
    }

    /// Runs a borrowed-world read only when the caller's document still owns this gateway
    /// identity. The operation uses one immutable lease, so replacement cannot redirect the
    /// document's callback to a new session.
    pub(crate) fn with_world_at_identity(
        &self,
        expected_identity: &GatewaySessionIdentity,
        read: &mut dyn FnMut(&World),
    ) -> Result<(), GatewayError> {
        let lease = self.current_lease();
        if lease.identity() != expected_identity {
            return Err(GatewayError::StaleGeneration {
                expected_generation: expected_identity.gateway_generation(),
                current_generation: lease.generation(),
            });
        }
        lease.gateway().with_world(read)
    }

    pub fn with_world_mut(&self, write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        self.current_lease().gateway().with_world_mut(write)
    }

    /// Mutable counterpart to `with_world_at_identity`.
    pub(crate) fn with_world_mut_at_identity(
        &self,
        expected_identity: &GatewaySessionIdentity,
        write: &mut dyn FnMut(&mut World),
    ) -> Result<(), GatewayError> {
        let lease = self.current_lease();
        if lease.identity() != expected_identity {
            return Err(GatewayError::StaleGeneration {
                expected_generation: expected_identity.gateway_generation(),
                current_generation: lease.generation(),
            });
        }
        lease.gateway().with_world_mut(write)
    }

    pub fn query_world(&self, query: WorldQuery) -> Result<WorldQueryResult, GatewayError> {
        self.current_lease().gateway().query_world(query)
    }

    /// Executes a serialized query only against the runtime identity captured by the caller.
    pub(crate) fn query_world_at_identity(
        &self,
        expected_identity: &GatewaySessionIdentity,
        query: WorldQuery,
    ) -> Result<WorldQueryResult, GatewayError> {
        let lease = self.current_lease();
        if lease.identity() != expected_identity {
            return Err(GatewayError::StaleGeneration {
                expected_generation: expected_identity.gateway_generation(),
                current_generation: lease.generation(),
            });
        }
        let result = lease.gateway().query_world(query)?;
        let current = self.current_lease();
        if current.identity() != expected_identity {
            return Err(GatewayError::StaleGeneration {
                expected_generation: expected_identity.gateway_generation(),
                current_generation: current.generation(),
            });
        }
        Ok(result)
    }

    pub(crate) fn watch_world(
        &self,
        registration: WatchRegistration,
    ) -> Result<WatchToken, GatewayError> {
        self.current_lease().gateway().watch_world(registration)
    }

    pub(crate) fn unwatch_world(&self, token: WatchToken) -> Result<bool, GatewayError> {
        self.current_lease().gateway().unwatch_world(token)
    }

    pub(crate) fn drain_world_invalidations(&self) -> Result<Vec<InvalidationBatch>, GatewayError> {
        self.current_lease().gateway().drain_world_invalidations()
    }

    pub fn tick_frame(&self) -> Result<EditorRuntimeFrameDemand, GatewayError> {
        self.current_lease().gateway().tick_frame()
    }

    pub fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), GatewayError> {
        self.current_lease().gateway().handle_event(event)
    }

    /// Dispatches one synchronous event only to the runtime generation selected by the caller.
    pub(crate) fn handle_event_at_identity(
        &self,
        expected_identity: &GatewaySessionIdentity,
        event: ZrRuntimeEventV1,
    ) -> Result<(), GatewayError> {
        let lease = self.current_lease();
        if lease.identity() != expected_identity {
            return Err(GatewayError::StaleGeneration {
                expected_generation: expected_identity.gateway_generation(),
                current_generation: lease.generation(),
            });
        }
        lease.gateway().handle_event(event)
    }

    pub fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<EditorRuntimeFrame, GatewayError> {
        self.current_lease().gateway().capture_frame(viewport, size)
    }

    /// Captures a viewport product only from the runtime origin selected by the caller.
    ///
    /// The immutable lease keeps replacement from redirecting the capture between the identity
    /// check and the runtime call. The returned frame therefore has a provenance that can be
    /// retained alongside asynchronous operations against that displayed product.
    pub(crate) fn capture_frame_at_identity(
        &self,
        expected_identity: &GatewaySessionIdentity,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<EditorRuntimeFrame, GatewayError> {
        let lease = self.current_lease();
        if lease.identity() != expected_identity {
            return Err(GatewayError::StaleGeneration {
                expected_generation: expected_identity.gateway_generation(),
                current_generation: lease.generation(),
            });
        }
        lease.gateway().capture_frame(viewport, size)
    }

    pub fn bind_viewport_surface(
        &self,
        request: ZrRuntimeBindViewportSurfaceRequestV1,
    ) -> Result<(), GatewayError> {
        self.current_lease()
            .gateway()
            .bind_viewport_surface(request)
    }

    pub fn unbind_viewport_surface(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<(), GatewayError> {
        self.current_lease()
            .gateway()
            .unbind_viewport_surface(viewport)
    }

    pub fn present_viewport(&self, request: ZrRuntimeFrameRequestV1) -> Result<(), GatewayError> {
        self.current_lease().gateway().present_viewport(request)
    }

    pub fn submit_highlight_set(&self, set: EditorRuntimeHighlightSet) -> Result<(), GatewayError> {
        self.current_lease().gateway().submit_highlight_set(set)
    }

    pub fn profile_control(
        &self,
        request: &ProfileControlRequest,
    ) -> Result<Option<ProfileControlResponse>, GatewayError> {
        self.current_lease().gateway().profile_control(request)
    }

    pub(crate) fn subscribe_plugin_event(
        &self,
        event_id: &str,
        payload_schema: &str,
    ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, GatewayError> {
        self.current_lease()
            .gateway()
            .subscribe_plugin_event(event_id, payload_schema)
    }

    pub(crate) fn unsubscribe_plugin_event(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, GatewayError> {
        self.current_lease()
            .gateway()
            .unsubscribe_plugin_event(subscription)
    }

    pub(crate) fn drain_plugin_events(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<EditorRuntimePluginEventPage, GatewayError> {
        self.current_lease()
            .gateway()
            .drain_plugin_events(subscription)
    }

    pub fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        self.current_lease().gateway().submit_operation(request)
    }

    pub fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        self.current_lease().gateway().poll_operation(handle)
    }

    pub fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        self.current_lease().gateway().harvest_operation(handle)
    }
}

impl EditorRuntimeGateway for EditorRuntimeGatewayHandle {
    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        EditorRuntimeGatewayHandle::capabilities(self)
    }

    fn module_composition_receipt(&self) -> Option<Arc<ZrRuntimeModuleCompositionReceiptV1>> {
        EditorRuntimeGatewayHandle::module_composition_receipt(self)
    }

    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        EditorRuntimeGatewayHandle::session_handle(self)
    }

    fn session_identity(&self) -> GatewaySessionIdentity {
        EditorRuntimeGatewayHandle::identity(self)
    }

    fn with_world(&self, read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
        EditorRuntimeGatewayHandle::with_world(self, read)
    }

    fn with_world_mut(&self, write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        EditorRuntimeGatewayHandle::with_world_mut(self, write)
    }

    fn query_world(&self, query: WorldQuery) -> Result<WorldQueryResult, GatewayError> {
        EditorRuntimeGatewayHandle::query_world(self, query)
    }

    fn watch_world(&self, registration: WatchRegistration) -> Result<WatchToken, GatewayError> {
        EditorRuntimeGatewayHandle::watch_world(self, registration)
    }

    fn unwatch_world(&self, token: WatchToken) -> Result<bool, GatewayError> {
        EditorRuntimeGatewayHandle::unwatch_world(self, token)
    }

    fn drain_world_invalidations(&self) -> Result<Vec<InvalidationBatch>, GatewayError> {
        EditorRuntimeGatewayHandle::drain_world_invalidations(self)
    }

    fn tick_frame(&self) -> Result<EditorRuntimeFrameDemand, GatewayError> {
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

    fn bind_viewport_surface(
        &self,
        request: ZrRuntimeBindViewportSurfaceRequestV1,
    ) -> Result<(), GatewayError> {
        EditorRuntimeGatewayHandle::bind_viewport_surface(self, request)
    }

    fn unbind_viewport_surface(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<(), GatewayError> {
        EditorRuntimeGatewayHandle::unbind_viewport_surface(self, viewport)
    }

    fn present_viewport(&self, request: ZrRuntimeFrameRequestV1) -> Result<(), GatewayError> {
        EditorRuntimeGatewayHandle::present_viewport(self, request)
    }

    fn submit_highlight_set(&self, set: EditorRuntimeHighlightSet) -> Result<(), GatewayError> {
        EditorRuntimeGatewayHandle::submit_highlight_set(self, set)
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
    ) -> Result<EditorRuntimePluginEventPage, GatewayError> {
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
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        EditorRuntimeGatewayHandle::poll_operation(self, handle)
    }

    fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        EditorRuntimeGatewayHandle::harvest_operation(self, handle)
    }
}

#[cfg(test)]
mod tests {
    use super::{next_generation, GatewayError};

    #[test]
    fn next_gateway_generation_returns_typed_error_at_u64_max() {
        assert_eq!(
            next_generation(u64::MAX),
            Err(GatewayError::GenerationExhausted)
        );
    }
}
