mod compiled_scene_outputs;
mod history;
mod post_process;
mod render;
mod runtime_prepare;
mod scene_passes;

pub(in crate::graphics::scene::scene_renderer::core) use compiled_scene_outputs::SceneRendererCompiledSceneOutputs;
pub(in crate::graphics::scene::scene_renderer::core) use render::{
    VirtualGeometryHardwareRasterizationPassStoreParts, VirtualGeometryIndirectStatsStoreParts,
    VirtualGeometryNodeAndClusterCullPassStoreParts, VirtualGeometryVisBuffer64PassStoreParts,
};
