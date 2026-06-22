use super::super::super::super::super::resources::depth_sampling_mode::PostProcessDepthSamplingMode;
use super::super::super::super::super::scene_post_process_resources::ScenePostProcessResources;

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_group_entries<'a>(
    resources: &'a ScenePostProcessResources,
    post_process_params_buffer: &'a wgpu::Buffer,
    scene_color_view: &'a wgpu::TextureView,
    scene_depth_view: &'a wgpu::TextureView,
    motion_vector_neighbor_max_view: &'a wgpu::TextureView,
    scene_normal_view: &'a wgpu::TextureView,
    scene_material_view: Option<&'a wgpu::TextureView>,
    ao_view: &'a wgpu::TextureView,
    contact_shadow_view: &'a wgpu::TextureView,
    previous_scene_color_view: Option<&'a wgpu::TextureView>,
    previous_global_illumination_view: Option<&'a wgpu::TextureView>,
    previous_screen_space_reflection_history_view: Option<&'a wgpu::TextureView>,
    resolved_screen_space_reflection_history_view: Option<&'a wgpu::TextureView>,
    screen_space_reflection_specular_occlusion_view: Option<&'a wgpu::TextureView>,
    screen_space_reflection_depth_pyramid_view: Option<&'a wgpu::TextureView>,
    screen_space_reflection_reflection_pyramid_view: Option<&'a wgpu::TextureView>,
    screen_space_reflection_depth_pyramid_coarse_view: Option<&'a wgpu::TextureView>,
    screen_space_reflection_reflection_pyramid_coarse_view: Option<&'a wgpu::TextureView>,
    bloom_view: &'a wgpu::TextureView,
    depth_of_field_coc_view: &'a wgpu::TextureView,
    depth_of_field_bokeh_view: &'a wgpu::TextureView,
    effect_lut_view: &'a wgpu::TextureView,
    effect_lut_3d_view: &'a wgpu::TextureView,
    cluster_buffer: &'a wgpu::Buffer,
    exposure_buffer: &'a wgpu::Buffer,
) -> [wgpu::BindGroupEntry<'a>; 29] {
    let scene_depth_binding_view = match resources.depth_sampling_mode {
        PostProcessDepthSamplingMode::RawDepthTexture => scene_depth_view,
        PostProcessDepthSamplingMode::ViewportDepthFallback => &resources.black_texture_view,
    };

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
            resource: post_process_params_buffer.as_entire_binding(),
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
        wgpu::BindGroupEntry {
            binding: 10,
            resource: wgpu::BindingResource::TextureView(effect_lut_view),
        },
        wgpu::BindGroupEntry {
            binding: 11,
            resource: wgpu::BindingResource::TextureView(scene_depth_binding_view),
        },
        wgpu::BindGroupEntry {
            binding: 12,
            resource: wgpu::BindingResource::TextureView(effect_lut_3d_view),
        },
        wgpu::BindGroupEntry {
            binding: 13,
            resource: wgpu::BindingResource::Sampler(&resources.effect_lut_sampler),
        },
        wgpu::BindGroupEntry {
            binding: 14,
            resource: wgpu::BindingResource::TextureView(scene_normal_view),
        },
        wgpu::BindGroupEntry {
            binding: 15,
            resource: wgpu::BindingResource::Sampler(&resources.scene_depth_sampler),
        },
        wgpu::BindGroupEntry {
            binding: 16,
            resource: wgpu::BindingResource::TextureView(
                scene_material_view.unwrap_or(&resources.black_texture_view),
            ),
        },
        wgpu::BindGroupEntry {
            binding: 17,
            resource: wgpu::BindingResource::TextureView(depth_of_field_coc_view),
        },
        wgpu::BindGroupEntry {
            binding: 18,
            resource: wgpu::BindingResource::TextureView(depth_of_field_bokeh_view),
        },
        wgpu::BindGroupEntry {
            binding: 19,
            resource: wgpu::BindingResource::TextureView(motion_vector_neighbor_max_view),
        },
        wgpu::BindGroupEntry {
            binding: 20,
            resource: wgpu::BindingResource::TextureView(
                previous_screen_space_reflection_history_view
                    .unwrap_or(&resources.black_texture_view),
            ),
        },
        wgpu::BindGroupEntry {
            binding: 21,
            resource: wgpu::BindingResource::TextureView(
                resolved_screen_space_reflection_history_view
                    .unwrap_or(&resources.black_texture_view),
            ),
        },
        wgpu::BindGroupEntry {
            binding: 22,
            resource: wgpu::BindingResource::TextureView(
                screen_space_reflection_specular_occlusion_view
                    .unwrap_or(&resources.white_texture_view),
            ),
        },
        wgpu::BindGroupEntry {
            binding: 23,
            resource: wgpu::BindingResource::TextureView(
                screen_space_reflection_depth_pyramid_view.unwrap_or(&resources.black_texture_view),
            ),
        },
        wgpu::BindGroupEntry {
            binding: 24,
            resource: wgpu::BindingResource::TextureView(
                screen_space_reflection_reflection_pyramid_view
                    .unwrap_or(&resources.black_texture_view),
            ),
        },
        wgpu::BindGroupEntry {
            binding: 25,
            resource: wgpu::BindingResource::TextureView(
                screen_space_reflection_depth_pyramid_coarse_view
                    .unwrap_or(&resources.black_texture_view),
            ),
        },
        wgpu::BindGroupEntry {
            binding: 26,
            resource: wgpu::BindingResource::TextureView(
                screen_space_reflection_reflection_pyramid_coarse_view
                    .unwrap_or(&resources.black_texture_view),
            ),
        },
        wgpu::BindGroupEntry {
            binding: 27,
            resource: wgpu::BindingResource::TextureView(contact_shadow_view),
        },
        wgpu::BindGroupEntry {
            binding: 28,
            resource: exposure_buffer.as_entire_binding(),
        },
    ]
}
