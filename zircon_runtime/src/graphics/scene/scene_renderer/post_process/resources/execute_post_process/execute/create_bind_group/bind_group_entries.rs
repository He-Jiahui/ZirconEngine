use super::super::super::super::super::scene_post_process_resources::ScenePostProcessResources;

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_group_entries<'a>(
    resources: &'a ScenePostProcessResources,
    scene_color_view: &'a wgpu::TextureView,
    ao_view: &'a wgpu::TextureView,
    previous_scene_color_view: Option<&'a wgpu::TextureView>,
    previous_global_illumination_view: Option<&'a wgpu::TextureView>,
    bloom_view: &'a wgpu::TextureView,
    cluster_buffer: &'a wgpu::Buffer,
) -> [wgpu::BindGroupEntry<'a>; 10] {
    [
        wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(scene_color_view),
        },
        wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::TextureView(ao_view),
        },
        wgpu::BindGroupEntry {
            binding: 2,
            resource: wgpu::BindingResource::TextureView(
                previous_scene_color_view.unwrap_or(&resources.black_texture_view),
            ),
        },
        wgpu::BindGroupEntry {
            binding: 3,
            resource: wgpu::BindingResource::TextureView(bloom_view),
        },
        wgpu::BindGroupEntry {
            binding: 4,
            resource: resources.post_process_params_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
            binding: 5,
            resource: cluster_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
            binding: 6,
            resource: resources.reflection_probe_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
            binding: 7,
            resource: resources.hybrid_gi_probe_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
            binding: 8,
            resource: resources.hybrid_gi_trace_region_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
            binding: 9,
            resource: wgpu::BindingResource::TextureView(
                previous_global_illumination_view.unwrap_or(&resources.black_texture_view),
            ),
        },
    ]
}
