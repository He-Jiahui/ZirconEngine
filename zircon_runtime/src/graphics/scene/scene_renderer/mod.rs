mod core;
mod deferred;
mod graph_execution;
mod history;
mod hybrid_gi;
mod mesh;
mod overlay;
mod particle;
mod post_process;
mod prepass;
mod primitives;
mod ui;
mod virtual_geometry;

pub use core::SceneRenderer;
pub(crate) use graph_execution::RenderPassExecutorId;

pub(crate) use core::{create_depth_texture, OFFSCREEN_FORMAT};
pub(crate) use deferred::GBUFFER_ALBEDO_FORMAT;
pub(crate) use hybrid_gi::{
    HybridGiGpuPendingReadback, HybridGiGpuReadback, HybridGiScenePrepareResourcesSnapshot,
};
#[cfg(test)]
pub(crate) use overlay::ViewportOverlayRenderer;
pub(crate) use post_process::{cluster_buffer_bytes_for_size, cluster_dimensions_for_size};
pub(crate) use prepass::NORMAL_FORMAT;
pub(crate) use virtual_geometry::{VirtualGeometryGpuPendingReadback, VirtualGeometryGpuReadback};
