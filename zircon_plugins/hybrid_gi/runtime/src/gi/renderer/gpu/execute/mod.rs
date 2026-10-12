#[path = "../../../gpu/execute/card_capture_shading.rs"]
mod card_capture_shading;
#[path = "collect_inputs.rs"]
mod collect_inputs;
#[path = "../../../gpu/execute/create_bind_group.rs"]
mod create_bind_group;
#[path = "../../../gpu/execute/create_buffers/mod.rs"]
mod create_buffers;
#[path = "dispatch.rs"]
mod dispatch;
#[path = "../../../execute/dispatch_probe_trace_tiles.rs"]
mod dispatch_probe_trace_tiles;
#[path = "../../../gpu/dispatch_radiance_cache/mod.rs"]
mod dispatch_radiance_cache;
#[path = "execute.rs"]
mod execute;
#[path = "../../../hybrid_gi_prepare_execution_buffers.rs"]
mod hybrid_gi_prepare_execution_buffers;
#[path = "../../../hybrid_gi_prepare_execution_inputs.rs"]
mod hybrid_gi_prepare_execution_inputs;
#[path = "../../../gpu/execute/material_capture_source.rs"]
mod material_capture_source;
#[path = "queue_params.rs"]
mod queue_params;
#[path = "../../../gpu/execute/voxel_clipmap_debug.rs"]
mod voxel_clipmap_debug;

pub(in crate::hybrid_gi::renderer::gpu_resources) use dispatch_probe_trace_tiles::{
    create_probe_trace_tile_dispatch_bind_group_layout, create_probe_trace_tile_dispatch_pipeline,
};
pub(in crate::hybrid_gi::renderer::gpu_resources) use dispatch_radiance_cache::{
    create_radiance_cache_atlas_buffer, create_radiance_cache_bind_group_layout,
    create_radiance_cache_consume_pipeline, create_radiance_cache_mark_buffer,
    create_radiance_cache_params_buffers, create_radiance_cache_storage_buffer,
    create_radiance_cache_update_pipeline,
};
pub(in crate::hybrid_gi::renderer) use material_capture_source::{
    HybridGiMaterialCaptureSeed, HybridGiMaterialCaptureSource, HybridGiMaterialCaptureTextureKey,
};
