pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_compute_executor;
pub(in crate::graphics) mod ibl_bake_graph_plan;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_runtime_writeback;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_shader_plan;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_binding;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_command_plan;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_dispatch;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_pipeline_cache;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_readback;
mod lightmap_binding;
mod probe_buffer;
pub(crate) mod procedural_environment;
pub(in crate::graphics) mod realtime_ibl_capture_wgpu;
pub(in crate::graphics) mod realtime_ibl_gpu_resources;
pub(in crate::graphics) mod realtime_ibl_gpu_timestamps;
pub(in crate::graphics) mod realtime_ibl_graph_plan;
pub(in crate::graphics) mod realtime_ibl_runtime;
pub(in crate::graphics) mod realtime_ibl_time_slice;
pub(in crate::graphics) mod realtime_ibl_wgpu_recorder;
mod scene_bind_group_layout;

pub(in crate::graphics::scene::scene_renderer) use ibl_bake_wgpu_pipeline_cache::IblBakeWgpuPipelineCache;
pub(in crate::graphics::scene::scene_renderer) use lightmap_binding::{
    LightmapGpuBindings, SceneLightmapResources, lightmap_bind_group_layout_entries,
};
pub(in crate::graphics::scene::scene_renderer) use probe_buffer::{
    PLANAR_REFLECTION_TEXTURE_SIZE, ReflectionProbeGpuBindings, SceneReflectionProbeResources,
    reflection_probe_bind_group_layout_entries,
};
pub use realtime_ibl_gpu_timestamps::RealtimeIblGpuTimingReport;
pub(in crate::graphics::scene::scene_renderer) use realtime_ibl_runtime::{
    RealtimeIblPendingSubmission, RealtimeIblPreparedFrame, RealtimeIblRuntime,
};
pub(in crate::graphics::scene::scene_renderer) use scene_bind_group_layout::scene_bind_group_layout_entries;
