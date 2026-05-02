mod core;
mod deferred;
mod graph_execution;
mod history;
mod mesh;
mod overlay;
mod particle;
mod post_process;
mod prepass;
mod primitives;
mod ui;

pub use core::SceneRenderer;
pub use graph_execution::{
    RenderPassExecutionContext, RenderPassExecutorFn, RenderPassExecutorId,
    RenderPassExecutorRegistration,
};

pub(crate) use core::{create_depth_texture, OFFSCREEN_FORMAT};
pub(crate) use deferred::GBUFFER_ALBEDO_FORMAT;
#[cfg(test)]
pub(crate) use overlay::ViewportOverlayRenderer;
pub(crate) use post_process::{cluster_buffer_bytes_for_size, cluster_dimensions_for_size};
pub(crate) use prepass::NORMAL_FORMAT;
