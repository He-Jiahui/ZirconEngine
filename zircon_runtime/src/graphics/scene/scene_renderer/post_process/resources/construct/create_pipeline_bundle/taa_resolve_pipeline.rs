use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;
use crate::graphics::scene::scene_renderer::post_process::POST_PROCESS_INTERMEDIATE_HDR_FORMAT;

const TAA_RESOLVE_SHADER: &str = include_str!("../../../../temporal/taa/shaders/taa_resolve.wgsl");
const TAA_OUTPUT_FORMAT: wgpu::TextureFormat = POST_PROCESS_INTERMEDIATE_HDR_FORMAT;
const TAA_HISTORY_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub(super) fn taa_resolve_pipeline(
    device: &wgpu::Device,
    taa_resolve_bind_group_layout: &wgpu::BindGroupLayout,
    depth_sampling_mode: PostProcessDepthSamplingMode,
) -> wgpu::RenderPipeline {
    let shader_source = depth_sampling_mode.taa_resolve_shader_source(TAA_RESOLVE_SHADER);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-taa-resolve-shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-taa-resolve-pipeline-layout"),
        bind_group_layouts: &[Some(taa_resolve_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-taa-resolve-pipeline"),
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
            entry_point: Some("fs_taa_resolve"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[
                Some(wgpu::ColorTargetState {
                    format: TAA_OUTPUT_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: TAA_HISTORY_FORMAT,
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
    use super::{
        PostProcessDepthSamplingMode, TAA_HISTORY_FORMAT, TAA_OUTPUT_FORMAT, TAA_RESOLVE_SHADER,
    };

    fn validate_shader_source(name: &str, shader_source: &str) {
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
    fn taa_resolve_shader_parses_and_declares_history_outputs() {
        validate_shader_source("taa_resolve.wgsl", TAA_RESOLVE_SHADER);
        assert_eq!(TAA_OUTPUT_FORMAT, wgpu::TextureFormat::Rg11b10Ufloat);
        assert_eq!(TAA_HISTORY_FORMAT, wgpu::TextureFormat::Rgba16Float);
        assert!(TAA_RESOLVE_SHADER.contains("@group(0) @binding(0) var scene_color_tex"));
        assert!(TAA_RESOLVE_SHADER.contains("@group(0) @binding(1) var scene_depth_tex"));
        assert!(TAA_RESOLVE_SHADER.contains("@group(0) @binding(2) var scene_velocity_tex"));
        assert!(TAA_RESOLVE_SHADER.contains("@group(0) @binding(3) var taa_history_previous_tex"));
        assert!(TAA_RESOLVE_SHADER.contains("@group(0) @binding(5) var taa_reactive_mask_tex"));
        assert!(TAA_RESOLVE_SHADER.contains("@location(0) resolved_scene_color"));
        assert!(TAA_RESOLVE_SHADER.contains("@location(1) current_history"));
        assert!(TAA_RESOLVE_SHADER.contains("rgb_to_ycocg"));
        assert!(TAA_RESOLVE_SHADER.contains("clip_towards_aabb_center"));
        assert!(TAA_RESOLVE_SHADER.contains("scene_color_neighborhood_ycocg_bounds"));
        assert!(TAA_RESOLVE_SHADER.contains("closest_depth_coord"));
        assert!(TAA_RESOLVE_SHADER.contains("reproject_history_coord"));
        assert!(TAA_RESOLVE_SHADER.contains("responsive_and_reactive"));
        assert!(TAA_RESOLVE_SHADER.contains("responsive_rejection"));
        assert!(TAA_RESOLVE_SHADER.contains("load_authored_reactive_mask"));
        assert!(TAA_RESOLVE_SHADER.contains("textureDimensions(taa_reactive_mask_tex)"));
        assert!(TAA_RESOLVE_SHADER.contains("let mask_coord = clamp(coord"));
    }

    #[test]
    fn taa_resolve_fallback_shader_parses_without_depth_texture_sampling() {
        let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
            .taa_resolve_shader_source(TAA_RESOLVE_SHADER);

        validate_shader_source("taa_resolve.viewport_fallback.wgsl", &shader_source);
        assert!(!shader_source.contains("texture_depth_2d"));
        assert!(!shader_source.contains("textureLoad(scene_depth_tex"));
    }
}
