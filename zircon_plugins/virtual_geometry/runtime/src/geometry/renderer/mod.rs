// 本模块保留插件侧 GPU/readback 投影；生产 pass executor 另在 render_pass_executors 注册。
#[path = "../gpu_readback/mod.rs"]
mod gpu_readback;
#[path = "gpu/mod.rs"]
mod gpu_resources;
#[path = "../root_mesh_sources/mod.rs"]
mod root_mesh_sources;
#[path = "outputs/mod.rs"]
mod root_output_sources;
#[path = "passes/mod.rs"]
mod root_render_passes;
#[path = "readbacks/mod.rs"]
mod root_state_readbacks;
#[path = "../../virtual_geometry_render_frame.rs"]
mod virtual_geometry_render_frame;

pub(crate) use gpu_readback::{
    VirtualGeometryGpuPendingReadback, VirtualGeometryGpuReadback,
    VirtualGeometryGpuReadbackCompletionParts, VirtualGeometryGpuReadbackFuture,
};
pub(crate) use gpu_resources::VirtualGeometryGpuResources;
pub(crate) use root_output_sources::runtime_prepare_renderer_outputs;
pub(in crate::virtual_geometry) use virtual_geometry_render_frame::VirtualGeometryRenderFrame;

#[cfg(test)]
#[path = "uploader_test_support.rs"]
mod uploader_test_support;
#[cfg(test)]
pub(crate) use uploader_test_support::{
    uploader_test_collector_registration, VirtualGeometryUploaderTestCollector,
};

pub(crate) use root_render_passes::prepare_virtual_geometry_node_and_cluster_cull;

pub(crate) use root_render_passes::{materialize_prepared_cull, MaterializedPreparedCull};
