use crate::graphics::scene::anti_alias::fxaa::FXAA_SHADER_ENTRY_POINT;

use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;

const POST_PROCESS_SHADER: &str = include_str!("../../../shaders/post_process.wgsl");

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
                    format: target_format,
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
    use super::{PostProcessDepthSamplingMode, POST_PROCESS_SHADER};

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
    fn post_process_shader_samples_bound_effect_lut_texture_3d() {
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(12) var effect_lut_3d_tex"));
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(13) var effect_lut_sampler"));
        assert!(POST_PROCESS_SHADER.contains("textureSampleLevel(effect_lut_3d_tex"));
        assert!(POST_PROCESS_SHADER.contains("effect_lut_sampler"));
        assert!(POST_PROCESS_SHADER
            .contains("clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)) * axis_max + vec3<f32>(0.5)"));
        assert!(POST_PROCESS_SHADER.contains("if (binding_mode == 3u)"));
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
    fn post_process_shader_samples_bound_scene_material_texture_for_ssr_roughness() {
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(16) var scene_material_tex"));
        assert!(POST_PROCESS_SHADER.contains("textureLoad(scene_material_tex"));
        assert!(POST_PROCESS_SHADER.contains("fn load_scene_material_roughness"));
        assert!(POST_PROCESS_SHADER
            .contains("let roughness = load_scene_material_roughness(coord_i32, viewport_size)"));
        assert!(POST_PROCESS_SHADER.contains("roughness_visibility"));
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
