//! Stable rendering facade contracts.

mod error;
mod handle;
mod server;
mod types;

pub use error::RenderServerError;
pub use handle::{
    resolve_render_server, FrameHistoryHandle, RenderPipelineHandle, RenderServerHandle,
    RenderViewportHandle, RENDER_SERVER_NAME,
};
pub use server::RenderServer;
pub use types::{
    CapturedFrame, RenderCapabilitySummary, RenderCommand, RenderFeatureQualitySettings,
    RenderQualityProfile, RenderQuery, RenderQueueCapability, RenderStats,
    RenderViewportDescriptor,
};

#[cfg(test)]
mod tests;
