use super::super::super::super::super::scene_post_process_resources::ScenePostProcessResources;
use super::bind_group_entries::bind_group_entries;

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn create_bind_group(
    resources: &ScenePostProcessResources,
    device: &wgpu::Device,
    scene_color_view: &wgpu::TextureView,
    ao_view: &wgpu::TextureView,
    previous_scene_color_view: Option<&wgpu::TextureView>,
    previous_global_illumination_view: Option<&wgpu::TextureView>,
    bloom_view: &wgpu::TextureView,
    cluster_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    let entries = bind_group_entries(
        resources,
        scene_color_view,
        ao_view,
        previous_scene_color_view,
        previous_global_illumination_view,
        bloom_view,
        cluster_buffer,
    );
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-post-process-bind-group"),
        layout: &resources.post_process_bind_group_layout,
        entries: &entries,
    })
}
