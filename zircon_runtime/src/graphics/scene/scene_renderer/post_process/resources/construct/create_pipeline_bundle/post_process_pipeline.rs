use crate::graphics::scene::anti_alias::fxaa::FXAA_SHADER_ENTRY_POINT;
use crate::graphics::scene::scene_renderer::post_process::POST_PROCESS_TONEMAPPED_FORMAT;

use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;
use super::super::super::shader_sources::POST_PROCESS_SHADER;

pub(super) fn post_process_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    post_process_bind_group_layout: &wgpu::BindGroupLayout,
    depth_sampling_mode: PostProcessDepthSamplingMode,
) -> wgpu::RenderPipeline {
    let shader_source = depth_sampling_mode.post_process_shader_source(POST_PROCESS_SHADER);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-post-process-shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-post-process-pipeline-layout"),
        bind_group_layouts: &[Some(post_process_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-post-process-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(FXAA_SHADER_ENTRY_POINT),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[
                Some(wgpu::ColorTargetState {
                    format: POST_PROCESS_TONEMAPPED_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
            ],
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    use super::super::super::super::shader_sources::POST_PROCESS_SHADER;
    use super::PostProcessDepthSamplingMode;

    fn validate_post_process_shader_source(name: &str, shader_source: &str) {
        let module = naga::front::wgsl::parse_str(shader_source)
            .unwrap_or_else(|error| panic!("{name}: {}", error.emit_to_string(shader_source)));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator
            .validate(&module)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }

    #[test]
    fn post_process_shader_parses_after_lut_binding_expansion() {
        validate_post_process_shader_source("post_process.wgsl", POST_PROCESS_SHADER);
    }

    #[test]
    fn post_process_shader_samples_bound_effect_lut_texture() {
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(10) var effect_lut_tex"));
        assert!(POST_PROCESS_SHADER.contains("textureLoad(effect_lut_tex"));
        assert!(POST_PROCESS_SHADER.contains("sample_effect_lut_2d_strip"));
        assert!(POST_PROCESS_SHADER
            .contains("mapped = mix(mapped, sample_effect_lut(mapped, params.effect_flags.y)"));
    }

    #[test]
    fn post_process_shader_samples_bound_contact_shadow_occlusion_texture() {
        assert!(POST_PROCESS_SHADER.contains("lighting_flags: vec4<u32>"));
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(27) var contact_shadow_tex"));
        assert!(POST_PROCESS_SHADER.contains("params.lighting_flags.x != 0u"));
        assert!(POST_PROCESS_SHADER.contains("textureLoad(contact_shadow_tex"));
        assert!(POST_PROCESS_SHADER
            .contains("occlusion_factor = occlusion_factor * max(contact_shadow"));
    }

    #[test]
    fn post_process_shader_multiplies_color_grading_by_resolved_exposure() {
        assert!(POST_PROCESS_SHADER
            .contains("@group(0) @binding(28) var<storage, read> exposure_buffer"));
        assert!(POST_PROCESS_SHADER
            .contains("let exposure = params.grading.x * max(exposure_buffer[0].x, 0.0);"));
    }

    #[test]
    fn post_process_shader_samples_bound_effect_lut_texture_3d() {
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(12) var effect_lut_3d_tex"));
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(13) var effect_lut_sampler"));
        assert!(POST_PROCESS_SHADER.contains("textureSampleLevel(effect_lut_3d_tex"));
        assert!(POST_PROCESS_SHADER.contains("effect_lut_sampler"));
        assert!(POST_PROCESS_SHADER
            .contains("clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)) * axis_max + vec3<f32>(0.5)"));
        assert!(POST_PROCESS_SHADER.contains("if (binding_mode == 3u)"));
        assert!(POST_PROCESS_SHADER.contains("if (params.effect_flags.y == 4u)"));
        assert!(POST_PROCESS_SHADER.contains("return sample_effect_lut_3d(color);"));
    }

    #[test]
    fn post_process_shader_samples_bound_scene_depth_texture() {
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(11) var scene_depth_tex"));
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(15) var scene_depth_sampler"));
        assert!(POST_PROCESS_SHADER.contains("textureSample(scene_depth_tex"));
        assert!(POST_PROCESS_SHADER.contains("linearize_scene_depth(load_scene_depth"));
        assert!(
            POST_PROCESS_SHADER.contains("load_scene_view_depth(vec2<i32>(coord), viewport_size)")
        );
        assert!(POST_PROCESS_SHADER.contains("fn trace_screen_space_reflection"));
        assert!(POST_PROCESS_SHADER.contains("let sample_depth = load_scene_view_depth"));
        assert!(POST_PROCESS_SHADER.contains("abs(sample_depth - ray_depth)"));
        assert!(POST_PROCESS_SHADER.contains("fn reconstruct_view_position"));
        assert!(POST_PROCESS_SHADER.contains("fn project_view_position_to_pixel"));
    }

    #[test]
    fn post_process_shader_uses_lens_bokeh_depth_of_field_kernel() {
        assert!(POST_PROCESS_SHADER.contains("effect_dof_lens: vec4<f32>"));
        assert!(POST_PROCESS_SHADER.contains("const DOF_BOKEH_SAMPLE_COUNT: u32 = 12u"));
        assert!(POST_PROCESS_SHADER.contains("fn depth_of_field_radius"));
        assert!(POST_PROCESS_SHADER.contains("fn bokeh_aperture_radius"));
        assert!(POST_PROCESS_SHADER.contains("fn sample_depth_of_field_bokeh"));
        assert!(POST_PROCESS_SHADER.contains("params.effect_dof_lens.x / 50.0"));
        assert!(POST_PROCESS_SHADER.contains("round(params.effect_dof_lens.z)"));
        assert!(POST_PROCESS_SHADER.contains("sample_index < DOF_BOKEH_SAMPLE_COUNT"));
    }

    #[test]
    fn post_process_shader_samples_bound_depth_of_field_coc_texture() {
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(17) var depth_of_field_coc_tex"));
        assert!(POST_PROCESS_SHADER.contains("textureLoad(depth_of_field_coc_tex"));
        assert!(POST_PROCESS_SHADER.contains("fn load_depth_of_field_coc"));
        assert!(POST_PROCESS_SHADER.contains("fn depth_of_field_coc_radius"));
        assert!(POST_PROCESS_SHADER.contains("fn dilated_depth_of_field_coc"));
        assert!(POST_PROCESS_SHADER.contains("let north_coc = load_depth_of_field_coc"));
        assert!(POST_PROCESS_SHADER.contains("let east_coc = load_depth_of_field_coc"));
        assert!(POST_PROCESS_SHADER.contains("let dilated_coc = dilated_depth_of_field_coc"));
        assert!(POST_PROCESS_SHADER.contains("let prepared_coc_radius = depth_of_field_coc_radius"));
        assert!(POST_PROCESS_SHADER.contains("max(prepared_coc_radius, scene_depth_radius)"));
    }

    #[test]
    fn post_process_shader_samples_bound_depth_of_field_bokeh_texture() {
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(18) var depth_of_field_bokeh_tex"));
        assert!(POST_PROCESS_SHADER.contains("textureLoad(depth_of_field_bokeh_tex"));
        assert!(POST_PROCESS_SHADER.contains("fn load_depth_of_field_coc"));
        assert!(POST_PROCESS_SHADER.contains("fn load_depth_of_field_bokeh_seed"));
        assert!(POST_PROCESS_SHADER.contains("fn depth_of_field_bokeh_layer_weight"));
        assert!(POST_PROCESS_SHADER.contains("let far_layer = min(center_coc.x, sample_coc.x)"));
        assert!(POST_PROCESS_SHADER.contains("let near_layer = sample_coc.y"));
        assert!(POST_PROCESS_SHADER.contains("fn sample_prepared_depth_of_field_bokeh"));
        assert!(POST_PROCESS_SHADER.contains("let sample_coc = load_depth_of_field_coc"));
        assert!(POST_PROCESS_SHADER.contains("seed.a * depth_of_field_bokeh_layer_weight"));
        assert!(POST_PROCESS_SHADER
            .contains("let prepared_bokeh = sample_prepared_depth_of_field_bokeh"));
        assert!(POST_PROCESS_SHADER.contains("prepared_bokeh.a"));
        assert!(POST_PROCESS_SHADER.contains("mix(procedural_bokeh, prepared_bokeh.rgb"));
    }

    #[test]
    fn post_process_shader_samples_reconstructed_motion_vector_for_motion_blur() {
        assert!(POST_PROCESS_SHADER
            .contains("@group(0) @binding(19) var motion_vector_neighbor_max_tex"));
        assert!(POST_PROCESS_SHADER.contains("effect_motion_blur: vec4<f32>"));
        assert!(POST_PROCESS_SHADER.contains("const MOTION_BLUR_MAX_SAMPLES: u32 = 32u"));
        assert!(POST_PROCESS_SHADER.contains("fn load_motion_vector_neighbor_max"));
        assert!(POST_PROCESS_SHADER.contains("textureLoad(motion_vector_neighbor_max_tex"));
        assert!(!POST_PROCESS_SHADER.contains("fn choose_dominant_motion_blur_vector"));
        assert!(!POST_PROCESS_SHADER.contains("fn dominant_scene_motion_vector"));
        assert!(POST_PROCESS_SHADER.contains("fn motion_blur_sample_weight"));
        assert!(POST_PROCESS_SHADER.contains("fn motion_blur_depth_visibility"));
        assert!(POST_PROCESS_SHADER.contains("fn apply_motion_blur_vector_gather"));
        assert!(POST_PROCESS_SHADER.contains("params.effect_motion_blur.x"));
        assert!(POST_PROCESS_SHADER.contains("params.effect_motion_blur.y"));
        assert!(POST_PROCESS_SHADER.contains("let motion_vector = load_motion_vector_neighbor_max"));
        assert!(POST_PROCESS_SHADER
            .contains("let center_depth = load_scene_view_depth(coord_i32, viewport_size)"));
        assert!(POST_PROCESS_SHADER.contains("motion_vector * shutter_fraction"));
        assert!(POST_PROCESS_SHADER.contains("sample_index <= MOTION_BLUR_MAX_SAMPLES"));
        assert!(POST_PROCESS_SHADER
            .contains("let sample_depth = load_scene_view_depth(sample_coord, viewport_size)"));
        assert!(
            POST_PROCESS_SHADER.contains("motion_blur_sample_weight(motion_vector, sample_motion)")
        );
        assert!(POST_PROCESS_SHADER
            .contains("* motion_blur_depth_visibility(center_depth, sample_depth)"));
        assert!(POST_PROCESS_SHADER.contains("color = apply_motion_blur_vector_gather"));
    }

    #[test]
    fn post_process_shader_applies_final_stack_stylistic_effects_after_temporal_resolve() {
        assert!(POST_PROCESS_SHADER.contains("effect_vignette_grain: vec4<f32>"));
        assert!(POST_PROCESS_SHADER.contains("effect_chromatic_fog: vec4<f32>"));
        assert!(POST_PROCESS_SHADER.contains("effect_fog_color: vec4<f32>"));
        assert!(POST_PROCESS_SHADER.contains("effect_dither_ssr: vec4<f32>"));
        assert!(POST_PROCESS_SHADER.contains("fn apply_chromatic_aberration"));
        assert!(POST_PROCESS_SHADER.contains("params.effect_chromatic_fog.x"));
        assert!(POST_PROCESS_SHADER.contains("params.effect_chromatic_fog.y"));
        assert!(POST_PROCESS_SHADER.contains("fn apply_effect_fog"));
        assert!(POST_PROCESS_SHADER.contains("normalized_view_depth(scene_depth)"));
        assert!(POST_PROCESS_SHADER.contains("params.effect_fog_color.rgb"));
        assert!(POST_PROCESS_SHADER.contains("fn apply_vignette"));
        assert!(POST_PROCESS_SHADER.contains("params.effect_vignette_grain.x"));
        assert!(POST_PROCESS_SHADER.contains("params.effect_vignette_grain.y"));
        assert!(POST_PROCESS_SHADER.contains("params.effect_vignette_grain.z"));
        assert!(POST_PROCESS_SHADER.contains("fn apply_grain_and_dither"));
        assert!(POST_PROCESS_SHADER
            .contains("params.effect_vignette_grain.w * max(params.effect_fog_color.w, 0.0)"));
        assert!(POST_PROCESS_SHADER.contains("params.effect_dither_ssr.x"));
        assert!(POST_PROCESS_SHADER.contains("params.effect_dither_ssr.y"));

        let chromatic = POST_PROCESS_SHADER
            .find("color = apply_chromatic_aberration")
            .unwrap();
        let scene_composite = POST_PROCESS_SHADER
            .find("color = apply_scene_composite")
            .unwrap();
        let vignette = POST_PROCESS_SHADER.find("color = apply_vignette").unwrap();
        let grain = POST_PROCESS_SHADER
            .find("color = apply_grain_and_dither")
            .unwrap();

        assert!(chromatic < scene_composite);
        assert!(scene_composite < vignette);
        assert!(vignette < grain);

        let scene_composite_body = POST_PROCESS_SHADER
            .split("fn apply_scene_composite")
            .nth(1)
            .unwrap();
        let ssr_resolve = scene_composite_body
            .find("let resolved_reflection =")
            .unwrap();
        let fog = scene_composite_body
            .find("composited = apply_effect_fog")
            .unwrap();
        assert!(ssr_resolve < fog);
    }

    #[test]
    fn post_process_shader_samples_bound_scene_normal_texture_for_ssr() {
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(14) var scene_normal_tex"));
        assert!(POST_PROCESS_SHADER.contains("textureLoad(scene_normal_tex"));
        assert!(POST_PROCESS_SHADER.contains("fn load_scene_normal"));
        assert!(POST_PROCESS_SHADER.contains("effect_view_x: vec4<f32>"));
        assert!(POST_PROCESS_SHADER.contains("effect_view_y: vec4<f32>"));
        assert!(POST_PROCESS_SHADER.contains("effect_view_z: vec4<f32>"));
        assert!(POST_PROCESS_SHADER.contains("fn world_normal_to_view_space"));
        assert!(POST_PROCESS_SHADER.contains("dot(params.effect_view_x.xyz, world_normal)"));
        assert!(POST_PROCESS_SHADER.contains("dot(params.effect_view_y.xyz, world_normal)"));
        assert!(POST_PROCESS_SHADER.contains("dot(params.effect_view_z.xyz, world_normal)"));
        assert!(POST_PROCESS_SHADER
            .contains("let normal = world_normal_to_view_space(load_scene_normal"));
        assert!(POST_PROCESS_SHADER.contains("reflect(view_direction, normal)"));
    }

    #[test]
    fn post_process_shader_ray_marches_ssr_with_bounds_and_edge_fade() {
        assert!(POST_PROCESS_SHADER.contains("fn trace_screen_space_reflection"));
        assert!(POST_PROCESS_SHADER.contains("for (var step_index = 1u"));
        assert!(POST_PROCESS_SHADER.contains("min(params.effect_flags.z, 128u)"));
        assert!(POST_PROCESS_SHADER.contains("params.effect_ssr_limits.x"));
        assert!(POST_PROCESS_SHADER.contains("params.effect_projection.x"));
        assert!(POST_PROCESS_SHADER.contains("project_view_position_to_pixel(ray_position"));
        assert!(POST_PROCESS_SHADER.contains("fn screen_edge_fade"));
        assert!(POST_PROCESS_SHADER.contains("traced_reflection.a"));
    }

    #[test]
    fn post_process_shader_refines_projected_ssr_hits() {
        assert!(POST_PROCESS_SHADER.contains("const SSR_HIT_REFINE_STEPS: u32 = 4u"));
        assert!(POST_PROCESS_SHADER.contains("fn sample_screen_space_reflection_hit"));
        assert!(POST_PROCESS_SHADER.contains("fn refine_screen_space_reflection_hit"));
        assert!(POST_PROCESS_SHADER
            .contains("for (var refine_index = 0u; refine_index < SSR_HIT_REFINE_STEPS"));
        assert!(
            POST_PROCESS_SHADER.contains("let refined_hit = refine_screen_space_reflection_hit")
        );
        assert!(POST_PROCESS_SHADER.contains("(candidate_hit.z > 0.0 && ray_direction.z < 0.0)"));
        assert!(POST_PROCESS_SHADER.contains("if (hit.w < 0.0)"));
    }

    #[test]
    fn post_process_shader_reprojects_ssr_history_with_motion_vectors() {
        assert!(POST_PROCESS_SHADER.contains("struct ColorNeighborhood"));
        assert!(POST_PROCESS_SHADER.contains("fn scene_rgb_neighborhood"));
        assert!(POST_PROCESS_SHADER.contains("fn reproject_ssr_history_coord"));
        assert!(POST_PROCESS_SHADER.contains("coord) - motion_vector * vec2<f32>(viewport_size)"));
        assert!(POST_PROCESS_SHADER.contains("fn sample_reprojected_ssr_history"));
        assert!(POST_PROCESS_SHADER.contains("params.feature_flags.z == 0u"));
        assert!(POST_PROCESS_SHADER.contains("clamp("));
        assert!(POST_PROCESS_SHADER.contains("round(history_pixel)"));
        assert!(POST_PROCESS_SHADER.contains("vec2<f32>(viewport_size) - vec2<f32>(1.0, 1.0)"));
        assert!(POST_PROCESS_SHADER
            .contains("@group(0) @binding(20) var history_screen_space_reflection_tex"));
        assert!(POST_PROCESS_SHADER
            .contains("@group(0) @binding(21) var resolved_screen_space_reflection_tex"));
        assert!(POST_PROCESS_SHADER.contains("textureLoad(history_screen_space_reflection_tex"));
        assert!(POST_PROCESS_SHADER.contains("textureLoad(resolved_screen_space_reflection_tex"));
        assert!(POST_PROCESS_SHADER.contains("clamp(history.rgb, neighborhood.minimum"));
        assert!(POST_PROCESS_SHADER.contains("clamp(history.a, 0.0, 1.0)"));
        assert!(POST_PROCESS_SHADER.contains("fn ssr_temporal_blend_weight"));
        assert!(POST_PROCESS_SHADER
            .contains("let temporal_blend_max = clamp(params.effect_ssr_limits.z, 0.0, 1.0)"));
        assert!(POST_PROCESS_SHADER
            .contains("temporal_blend_max * traced_visibility * motion_stability"));
        assert!(POST_PROCESS_SHADER.contains("let motion_vector = load_motion_vector_neighbor_max"));
        assert!(POST_PROCESS_SHADER.contains("fn resolve_screen_space_reflection_history"));
        assert!(POST_PROCESS_SHADER.contains("fn fs_screen_space_reflection_resolve"));
        assert!(!POST_PROCESS_SHADER.contains("@location(2) screen_space_reflection_history"));
        assert!(POST_PROCESS_SHADER
            .contains("load_resolved_screen_space_reflection(coord_i32, viewport_size)"));
        assert!(
            !POST_PROCESS_SHADER.contains("screen_space_reflection_history = resolved_reflection")
        );
        assert!(!POST_PROCESS_SHADER.contains("fn apply_screen_space_reflection_seed"));
        assert!(
            POST_PROCESS_SHADER.contains("let temporal_history = sample_reprojected_ssr_history")
        );
        assert!(POST_PROCESS_SHADER
            .contains("mix(traced_reflection.rgb, temporal_history.rgb, temporal_weight)"));
    }

    #[test]
    fn post_process_shader_attenuates_ssr_with_ambient_occlusion() {
        assert!(POST_PROCESS_SHADER.contains("fn load_screen_space_reflection_ambient_occlusion"));
        assert!(POST_PROCESS_SHADER.contains("textureLoad(ambient_occlusion_tex"));
        assert!(
            POST_PROCESS_SHADER.contains("fn screen_space_reflection_specular_occlusion_factors")
        );
        assert!(POST_PROCESS_SHADER.contains("let ambient_occlusion ="));
        assert!(POST_PROCESS_SHADER.contains("let occlusion_response ="));
        assert!(POST_PROCESS_SHADER
            .contains("@group(0) @binding(22) var screen_space_reflection_specular_occlusion_tex"));
        assert!(POST_PROCESS_SHADER.contains("fn load_screen_space_reflection_specular_occlusion"));
        assert!(POST_PROCESS_SHADER
            .contains("textureLoad(screen_space_reflection_specular_occlusion_tex"));
        assert!(
            POST_PROCESS_SHADER.contains("clamp(factors.g, 0.0, 1.0) * clamp(traced_visibility")
        );
        assert!(POST_PROCESS_SHADER.contains("fn fs_screen_space_reflection_specular_occlusion"));
        assert!(POST_PROCESS_SHADER.contains("let specular_occlusion ="));
        assert!(POST_PROCESS_SHADER.contains("* specular_occlusion * 0.18"));
    }

    #[test]
    fn post_process_shader_samples_bound_scene_material_texture_for_ssr_roughness() {
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(16) var scene_material_tex"));
        assert!(POST_PROCESS_SHADER.contains("textureLoad(scene_material_tex"));
        assert!(POST_PROCESS_SHADER.contains("fn load_scene_material_roughness"));
        assert!(POST_PROCESS_SHADER
            .contains("let roughness = load_scene_material_roughness(coord_i32, viewport_size)"));
        assert!(POST_PROCESS_SHADER.contains("roughness_visibility"));
    }

    #[test]
    fn post_process_shader_samples_shared_hzb_and_reflection_pyramids_with_coarse_fallbacks() {
        assert!(POST_PROCESS_SHADER
            .contains("@group(0) @binding(23) var screen_space_reflection_depth_pyramid_tex"));
        assert!(POST_PROCESS_SHADER
            .contains("@group(0) @binding(24) var screen_space_reflection_reflection_pyramid_tex"));
        assert!(POST_PROCESS_SHADER.contains(
            "@group(0) @binding(25) var screen_space_reflection_depth_pyramid_coarse_tex"
        ));
        assert!(POST_PROCESS_SHADER.contains(
            "@group(0) @binding(26) var screen_space_reflection_reflection_pyramid_coarse_tex"
        ));
        assert!(POST_PROCESS_SHADER
            .contains("textureNumLevels(screen_space_reflection_depth_pyramid_tex)"));
        assert!(POST_PROCESS_SHADER
            .contains("textureNumLevels(screen_space_reflection_reflection_pyramid_tex)"));
        assert!(POST_PROCESS_SHADER.contains(
            "textureLoad(\n        screen_space_reflection_depth_pyramid_tex,\n        vec2<i32>(safe_coord),\n        mip_level"
        ));
        assert!(POST_PROCESS_SHADER.contains(
            "textureLoad(\n        screen_space_reflection_reflection_pyramid_tex,\n        vec2<i32>(safe_coord),\n        mip_level"
        ));
        assert!(POST_PROCESS_SHADER
            .contains("return load_screen_space_reflection_depth_pyramid_mip(coord, 1u)"));
        assert!(POST_PROCESS_SHADER
            .contains("return load_screen_space_reflection_reflection_pyramid_mip(coord, 1u)"));
        assert!(POST_PROCESS_SHADER
            .contains("return load_screen_space_reflection_depth_pyramid_mip(coord, 0u)"));
        assert!(POST_PROCESS_SHADER.contains(
            "textureLoad(\n        screen_space_reflection_reflection_pyramid_coarse_tex"
        ));
        assert!(POST_PROCESS_SHADER.contains("fn screen_space_reflection_depth_pyramid_trace_mip"));
        assert!(
            POST_PROCESS_SHADER.contains("fn screen_space_reflection_reflection_pyramid_rough_mip")
        );
        assert!(POST_PROCESS_SHADER
            .contains("let biased_roughness = clamp(roughness + params.effect_ssr_limits.w"));
        assert!(POST_PROCESS_SHADER.contains("smoothstep(0.2, 0.95, biased_roughness)"));
        assert!(POST_PROCESS_SHADER.contains("smoothstep(0.18, 1.0, biased_roughness)"));
    }

    #[test]
    fn post_process_viewport_depth_fallback_shader_parses_for_gl_backends() {
        let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
            .post_process_shader_source(POST_PROCESS_SHADER);

        validate_post_process_shader_source(
            "post_process.viewport_depth_fallback.wgsl",
            &shader_source,
        );
        assert!(!shader_source.contains("texture_depth_2d"));
        assert!(!shader_source.contains("textureSample(scene_depth_tex"));
        assert!(
            shader_source.contains("@group(0) @binding(11) var scene_depth_tex: texture_2d<f32>;")
        );
        assert!(shader_source.contains("return clamp(uv.y, 0.0, 1.0);"));
    }
}
