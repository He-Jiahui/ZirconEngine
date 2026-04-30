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
pub(in crate::graphics::scene::scene_renderer) use hybrid_gi::HybridGiGpuPendingReadback;
#[cfg(test)]
pub(crate) use hybrid_gi::HybridGiGpuReadback;
#[cfg(not(test))]
pub(in crate::graphics::scene::scene_renderer) use hybrid_gi::HybridGiGpuReadback;
pub(in crate::graphics) use hybrid_gi::HybridGiGpuReadbackCompletionParts;
#[cfg(test)]
pub(crate) use hybrid_gi::HybridGiScenePrepareResourcesSnapshot;
#[cfg(not(test))]
pub(in crate::graphics::scene::scene_renderer) use hybrid_gi::HybridGiScenePrepareResourcesSnapshot;
#[cfg(test)]
pub(crate) use overlay::ViewportOverlayRenderer;
pub(crate) use post_process::{cluster_buffer_bytes_for_size, cluster_dimensions_for_size};
pub(crate) use prepass::NORMAL_FORMAT;
pub(in crate::graphics::scene::scene_renderer) use virtual_geometry::VirtualGeometryGpuPendingReadback;
#[cfg(test)]
pub(crate) use virtual_geometry::VirtualGeometryGpuReadback;
#[cfg(not(test))]
pub(in crate::graphics::scene::scene_renderer) use virtual_geometry::VirtualGeometryGpuReadback;
pub(in crate::graphics) use virtual_geometry::VirtualGeometryGpuReadbackCompletionParts;
