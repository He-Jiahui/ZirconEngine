use std::sync::Arc;

use zircon_core::{CoreError, CoreHandle};

use crate::server::RenderServer;

pub const RENDER_SERVER_NAME: &str = "GraphicsModule.Manager.RenderServer";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderViewportHandle(u64);

impl RenderViewportHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderPipelineHandle(u64);

impl RenderPipelineHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FrameHistoryHandle(u64);

impl FrameHistoryHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone)]
pub struct RenderServerHandle {
    inner: Arc<dyn RenderServer>,
}

impl RenderServerHandle {
    pub fn new(inner: Arc<dyn RenderServer>) -> Self {
        Self { inner }
    }

    pub fn shared(&self) -> Arc<dyn RenderServer> {
        self.inner.clone()
    }
}

impl std::fmt::Debug for RenderServerHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderServerHandle").finish()
    }
}

pub fn resolve_render_server(core: &CoreHandle) -> Result<Arc<dyn RenderServer>, CoreError> {
    core.resolve_manager::<RenderServerHandle>(RENDER_SERVER_NAME)
        .map(|handle| handle.shared())
}
