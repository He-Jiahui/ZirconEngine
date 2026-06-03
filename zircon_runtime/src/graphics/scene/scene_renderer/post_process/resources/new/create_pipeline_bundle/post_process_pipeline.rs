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

    #[test]
    fn post_process_shader_parses_after_lut_binding_expansion() {
        naga::front::wgsl::parse_str(POST_PROCESS_SHADER).expect("post-process shader must parse");
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
        assert!(POST_PROCESS_SHADER.contains("abs(reflected_depth - current_depth)"));
    }

    #[test]
    fn post_process_shader_samples_bound_scene_normal_texture_for_ssr() {
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(14) var scene_normal_tex"));
        assert!(POST_PROCESS_SHADER.contains("textureLoad(scene_normal_tex"));
        assert!(POST_PROCESS_SHADER.contains("fn load_scene_normal"));
        assert!(POST_PROCESS_SHADER.contains("let normal = load_scene_normal(coord_i32"));
        assert!(POST_PROCESS_SHADER.contains("reflect(view_direction, normal)"));
    }

    #[test]
    fn post_process_viewport_depth_fallback_shader_parses_for_gl_backends() {
        let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
            .post_process_shader_source(POST_PROCESS_SHADER);

        naga::front::wgsl::parse_str(&shader_source)
            .expect("fallback post-process shader must parse");
        assert!(!shader_source.contains("texture_depth_2d"));
        assert!(!shader_source.contains("textureSample(scene_depth_tex"));
        assert!(
            shader_source.contains("@group(0) @binding(11) var scene_depth_tex: texture_2d<f32>;")
        );
        assert!(shader_source.contains("return clamp(uv.y, 0.0, 1.0);"));
    }
}
