use std::sync::{Arc, Mutex, MutexGuard};

use arc_swap::{ArcSwap, Guard};

use zircon_runtime::scene::World;
use zircon_runtime_interface::{
    ProfileControlRequest, ProfileControlResponse, ZrRuntimeEventV1, ZrRuntimeOperationHandle,
    ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1,
};

use super::{
    DetachedEditorRuntimeGateway, EditorRuntimeFrame, EditorRuntimeFrameDemand,
    EditorRuntimeGateway, EditorRuntimeHighlightSet, EditorRuntimePluginEventPage, GatewayError,
    RuntimeCapabilities, SharedEditorRuntimeGateway,
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
    gateway: SharedEditorRuntimeGateway,
    capabilities: Arc<RuntimeCapabilities>,
}

impl GatewayGeneration {
    fn new(id: u64, gateway: SharedEditorRuntimeGateway) -> Self {
        let capabilities = gateway.capabilities();
        Self {
            id,
            gateway,
            capabilities,
        }
    }
}

impl EditorRuntimeGatewayHandle {
    pub fn new(gateway: SharedEditorRuntimeGateway) -> Self {
        Self {
            inner: Arc::new(GatewayOwner {
                current: ArcSwap::from_pointee(GatewayGeneration::new(0, gateway)),
                replacement: Mutex::new(()),
            }),
        }
    }

    pub fn detached() -> Self {
        Self::new(Arc::new(DetachedEditorRuntimeGateway))
    }

    pub fn replace(&self, gateway: SharedEditorRuntimeGateway) -> Result<(), GatewayError> {
        let _replacement = self.replacement_lock();
        let next_generation = next_generation(self.generation_snapshot().id)?;
        self.inner
            .current
            .store(Arc::new(GatewayGeneration::new(next_generation, gateway)));
        Ok(())
    }

    pub fn generation(&self) -> u64 {
        self.generation_snapshot().id
    }

    fn generation_snapshot(&self) -> Guard<Arc<GatewayGeneration>> {
        self.inner.current.load()
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
        self.generation_snapshot().capabilities.clone()
    }

    pub fn session_handle(&self) -> ZrRuntimeSessionHandle {
        self.generation_snapshot().gateway.session_handle()
    }

    pub fn with_world(&self, read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
        self.generation_snapshot().gateway.with_world(read)
    }

    pub fn with_world_mut(&self, write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        self.generation_snapshot().gateway.with_world_mut(write)
    }

    pub fn tick_frame(&self) -> Result<EditorRuntimeFrameDemand, GatewayError> {
        self.generation_snapshot().gateway.tick_frame()
    }

    pub fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), GatewayError> {
        self.generation_snapshot().gateway.handle_event(event)
    }

    pub fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<EditorRuntimeFrame, GatewayError> {
        self.generation_snapshot()
            .gateway
            .capture_frame(viewport, size)
    }

    pub fn submit_highlight_set(&self, set: EditorRuntimeHighlightSet) -> Result<(), GatewayError> {
        self.generation_snapshot().gateway.submit_highlight_set(set)
    }

    pub fn profile_control(
        &self,
        request: &ProfileControlRequest,
    ) -> Result<Option<ProfileControlResponse>, GatewayError> {
        self.generation_snapshot().gateway.profile_control(request)
    }

    pub fn subscribe_plugin_event(
        &self,
        event_id: &str,
        payload_schema: &str,
    ) -> Result<Option<ZrRuntimePluginEventSubscriptionHandle>, GatewayError> {
        self.generation_snapshot()
            .gateway
            .subscribe_plugin_event(event_id, payload_schema)
    }

    pub fn unsubscribe_plugin_event(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<bool, GatewayError> {
        self.generation_snapshot()
            .gateway
            .unsubscribe_plugin_event(subscription)
    }

    pub fn drain_plugin_events(
        &self,
        subscription: ZrRuntimePluginEventSubscriptionHandle,
    ) -> Result<EditorRuntimePluginEventPage, GatewayError> {
        self.generation_snapshot()
            .gateway
            .drain_plugin_events(subscription)
    }

    pub fn submit_operation(
        &self,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationHandle, GatewayError> {
        self.generation_snapshot().gateway.submit_operation(request)
    }

    pub fn poll_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationStatusV2, GatewayError> {
        self.generation_snapshot().gateway.poll_operation(handle)
    }

    pub fn harvest_operation(
        &self,
        handle: ZrRuntimeOperationHandle,
    ) -> Result<ZrRuntimeOperationResultV1, GatewayError> {
        self.generation_snapshot().gateway.harvest_operation(handle)
    }
}

impl EditorRuntimeGateway for EditorRuntimeGatewayHandle {
    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
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
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    use arc_swap::ArcSwap;
    use zircon_runtime_interface::{ZrRuntimeSessionHandle, ZrRuntimeViewportHandle};

    use super::{
        DetachedEditorRuntimeGateway, EditorRuntimeGateway, EditorRuntimeGatewayHandle,
        EditorRuntimeHighlightSet, GatewayError, GatewayGeneration, GatewayOwner, next_generation,
    };

    #[derive(Default)]
    struct HighlightRecordingGateway {
        submissions: AtomicUsize,
    }

    impl EditorRuntimeGateway for HighlightRecordingGateway {
        fn session_handle(&self) -> ZrRuntimeSessionHandle {
            ZrRuntimeSessionHandle::invalid()
        }

        fn submit_highlight_set(
            &self,
            _set: EditorRuntimeHighlightSet,
        ) -> Result<(), GatewayError> {
            self.submissions.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn submit_through_generic<T: EditorRuntimeGateway>(gateway: &T) -> Result<(), GatewayError> {
        gateway.submit_highlight_set(EditorRuntimeHighlightSet::new(
            ZrRuntimeViewportHandle::new(4),
            1,
            [8, 2],
            true,
            [0.2, 0.6, 0.9, 1.0],
        ))
    }

    #[test]
    fn next_gateway_generation_returns_typed_error_at_u64_max() {
        assert_eq!(
            next_generation(u64::MAX),
            Err(GatewayError::GenerationExhausted)
        );
    }

    #[test]
    fn replacement_at_generation_limit_does_not_publish_a_new_gateway() {
        let handle = EditorRuntimeGatewayHandle {
            inner: Arc::new(GatewayOwner {
                current: ArcSwap::from_pointee(GatewayGeneration::new(
                    u64::MAX,
                    Arc::new(DetachedEditorRuntimeGateway),
                )),
                replacement: Mutex::new(()),
            }),
        };

        assert_eq!(
            handle.replace(Arc::new(DetachedEditorRuntimeGateway)),
            Err(GatewayError::GenerationExhausted)
        );
        assert_eq!(handle.generation(), u64::MAX);
    }

    #[test]
    fn highlight_submission_forwards_through_trait_object_and_generic_handle_dispatch() {
        let gateway = Arc::new(HighlightRecordingGateway::default());
        let handle = EditorRuntimeGatewayHandle::new(gateway.clone());

        let trait_object: &dyn EditorRuntimeGateway = &handle;
        trait_object
            .submit_highlight_set(EditorRuntimeHighlightSet::new(
                ZrRuntimeViewportHandle::new(3),
                2,
                [6, 1],
                true,
                [0.2, 0.6, 0.9, 1.0],
            ))
            .unwrap();
        submit_through_generic(&handle).unwrap();

        assert_eq!(gateway.submissions.load(Ordering::SeqCst), 2);
    }
}
