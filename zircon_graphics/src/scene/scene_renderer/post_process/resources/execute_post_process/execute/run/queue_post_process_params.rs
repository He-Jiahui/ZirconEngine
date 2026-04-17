use super::super::super::super::super::post_process_params::PostProcessParams;
use super::super::super::super::super::scene_post_process_resources::ScenePostProcessResources;

pub(super) fn queue_post_process_params(
    resources: &ScenePostProcessResources,
    queue: &wgpu::Queue,
    params: &PostProcessParams,
) {
    queue.write_buffer(
        &resources.post_process_params_buffer,
        0,
        bytemuck::bytes_of(params),
    );
}
