use std::cell::Cell;
use std::sync::Arc;

use zircon_runtime::core::framework::render::{HighlightRenderAttributes, HighlightSet};
use zircon_runtime::core::CoreHandle;
use zircon_runtime::scene::{LevelSystem, World};
use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchRegistration, WatchToken, WorldQuery, WorldQueryResult,
};
use zircon_runtime_interface::{
    ZrRuntimeOperationHandle, ZrRuntimeOperationResultV1, ZrRuntimeOperationStatusV2,
    ZrRuntimeOperationSubmitRequestV1, ZrRuntimeSessionHandle,
};

use super::{
    EditorRuntimeGateway, EditorRuntimeHighlightSet, GatewayError, GatewaySessionIdentity,
    RuntimeCapabilities,
};

thread_local! {
    static BORROWED_WORLD_CALLBACK_ACTIVE: Cell<bool> = const { Cell::new(false) };
}

struct BorrowedWorldCallbackGuard;

impl BorrowedWorldCallbackGuard {
    fn enter() -> Result<Self, GatewayError> {
        BORROWED_WORLD_CALLBACK_ACTIVE.with(|active| {
            if active.replace(true) {
                Err(GatewayError::ReentrantBorrowedWorldAccess)
            } else {
                Ok(Self)
            }
        })
    }
}

impl Drop for BorrowedWorldCallbackGuard {
    fn drop(&mut self) {
        BORROWED_WORLD_CALLBACK_ACTIVE.with(|active| active.set(false));
    }
}

#[derive(Clone, Debug)]
pub struct InProcessGateway {
    _core: Option<CoreHandle>,
    level: LevelSystem,
    capabilities: Arc<RuntimeCapabilities>,
}

impl InProcessGateway {
    pub fn new(core: CoreHandle, level: LevelSystem) -> Self {
        Self {
            _core: Some(core),
            level,
            capabilities: Arc::new(RuntimeCapabilities::editor_default()),
        }
    }

    /// Creates the stable authoring facade for an editor-owned level.
    pub fn for_authoring_level(level: LevelSystem) -> Self {
        Self {
            _core: None,
            level,
            capabilities: Arc::new(RuntimeCapabilities::editor_default()),
        }
    }
}

impl EditorRuntimeGateway for InProcessGateway {
    fn capabilities(&self) -> Arc<RuntimeCapabilities> {
        self.capabilities.clone()
    }

    fn session_handle(&self) -> ZrRuntimeSessionHandle {
        ZrRuntimeSessionHandle::invalid()
    }

    fn session_identity(&self) -> GatewaySessionIdentity {
        GatewaySessionIdentity::detached()
    }

    fn with_world(&self, read: &mut dyn FnMut(&World)) -> Result<(), GatewayError> {
        let _callback_guard = BorrowedWorldCallbackGuard::enter()?;
        self.level.with_world(read);
        Ok(())
    }

    fn with_world_mut(&self, write: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError> {
        let _callback_guard = BorrowedWorldCallbackGuard::enter()?;
        self.level.with_world_mut(write);
        Ok(())
    }

    fn query_world(&self, query: WorldQuery) -> Result<WorldQueryResult, GatewayError> {
        Ok(self.level.query_world(&query))
    }

    fn watch_world(&self, registration: WatchRegistration) -> Result<WatchToken, GatewayError> {
        Ok(self.level.watch_world(registration))
    }

    fn unwatch_world(&self, token: WatchToken) -> Result<bool, GatewayError> {
        Ok(self.level.unwatch_world(token))
    }

    fn drain_world_invalidations(&self) -> Result<Vec<InvalidationBatch>, GatewayError> {
        Ok(self.level.drain_world_invalidations())
    }

    fn submit_highlight_set(&self, set: EditorRuntimeHighlightSet) -> Result<(), GatewayError> {
        if !set.is_valid() {
            return Err(GatewayError::Protocol {
                message: "invalid runtime highlight set".to_owned(),
            });
        }

        self.level.submit_highlight_set(
            set.viewport().raw(),
            set.generation(),
            HighlightSet::new(
                set.entities().iter().copied(),
                HighlightRenderAttributes {
                    outline_enabled: set.outline_enabled(),
                    tint_rgba: set.tint_rgba(),
                },
            ),
        );
        Ok(())
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
