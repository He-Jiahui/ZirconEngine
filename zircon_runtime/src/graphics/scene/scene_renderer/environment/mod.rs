pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_compute_executor;
pub(in crate::graphics) mod ibl_bake_graph_plan;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_runtime_writeback;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_shader_plan;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_binding;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_command_plan;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_dispatch;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_pipeline_cache;
pub(in crate::graphics::scene::scene_renderer) mod ibl_bake_wgpu_readback;
mod probe_buffer;
pub(crate) mod procedural_environment;
mod scene_bind_group_layout;

pub(in crate::graphics::scene::scene_renderer) use ibl_bake_wgpu_pipeline_cache::IblBakeWgpuPipelineCache;
pub(in crate::graphics::scene::scene_renderer) use probe_buffer::{
    reflection_probe_bind_group_layout_entries, ReflectionProbeGpuBindings,
    SceneReflectionProbeResources,
};
pub(in crate::graphics::scene::scene_renderer) use scene_bind_group_layout::scene_bind_group_layout_entries;
