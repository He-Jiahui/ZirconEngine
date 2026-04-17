use super::super::scene_renderer::SceneRenderer;

pub(in crate::scene::scene_renderer::core) fn reset_last_runtime_outputs(
    renderer: &mut SceneRenderer,
) {
    renderer.last_hybrid_gi_gpu_readback = None;
    renderer.last_virtual_geometry_gpu_readback = None;
    renderer.last_virtual_geometry_indirect_draw_count = 0;
    renderer.last_virtual_geometry_indirect_buffer_count = 0;
    renderer.last_virtual_geometry_indirect_segment_count = 0;
    renderer.last_virtual_geometry_indirect_args_buffer = None;
    renderer.last_virtual_geometry_indirect_args_count = 0;
}
