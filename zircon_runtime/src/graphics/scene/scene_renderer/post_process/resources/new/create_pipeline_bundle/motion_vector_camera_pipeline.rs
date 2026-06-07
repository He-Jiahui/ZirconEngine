use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;

const MOTION_VECTOR_CAMERA_SHADER: &str =
    include_str!("../../../shaders/motion_vector_camera.wgsl");
const MOTION_VECTOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub(super) fn motion_vector_camera_pipeline(
    device: &wgpu::Device,
    motion_vector_camera_bind_group_layout: &wgpu::BindGroupLayout,
    depth_sampling_mode: PostProcessDepthSamplingMode,
) -> wgpu::RenderPipeline {
    let shader_source =
        depth_sampling_mode.motion_vector_camera_shader_source(MOTION_VECTOR_CAMERA_SHADER);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-motion-vector-camera-shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-motion-vector-camera-pipeline-layout"),
        bind_group_layouts: &[Some(motion_vector_camera_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-motion-vector-camera-pipeline"),
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
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: MOTION_VECTOR_FORMAT,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    use super::{PostProcessDepthSamplingMode, MOTION_VECTOR_CAMERA_SHADER};

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
    fn motion_vector_camera_shader_parses_and_reconstructs_previous_uv() {
        validate_shader_source("motion_vector_camera.wgsl", MOTION_VECTOR_CAMERA_SHADER);
        assert!(MOTION_VECTOR_CAMERA_SHADER.contains("texture_depth_2d"));
        assert!(MOTION_VECTOR_CAMERA_SHADER.contains("fn load_motion_vector_scene_depth"));
        assert!(MOTION_VECTOR_CAMERA_SHADER.contains("fn clip_to_uv"));
        assert!(MOTION_VECTOR_CAMERA_SHADER.contains("fn motion_vector_camera_velocity"));
        assert!(MOTION_VECTOR_CAMERA_SHADER.contains("current_world_from_clip"));
        assert!(MOTION_VECTOR_CAMERA_SHADER.contains("previous_clip_from_world"));
    }

    #[test]
    fn motion_vector_camera_fallback_shader_parses_without_depth_texture_sampling() {
        let shader_source = PostProcessDepthSamplingMode::ViewportDepthFallback
            .motion_vector_camera_shader_source(MOTION_VECTOR_CAMERA_SHADER);

        validate_shader_source(
            "motion_vector_camera.viewport_fallback.wgsl",
            &shader_source,
        );
        assert!(!shader_source.contains("texture_depth_2d"));
        assert!(!shader_source.contains("textureLoad(scene_depth_tex"));
    }
}
