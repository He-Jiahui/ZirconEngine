const MOTION_VECTOR_TILE_MAX_SHADER: &str =
    include_str!("../../../shaders/motion_vector_tile_max.wgsl");
const MOTION_VECTOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub(super) fn motion_vector_tile_max_pipeline(
    device: &wgpu::Device,
    motion_vector_tile_max_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-motion-vector-tile-max-shader"),
        source: wgpu::ShaderSource::Wgsl(MOTION_VECTOR_TILE_MAX_SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-motion-vector-tile-max-pipeline-layout"),
        bind_group_layouts: &[Some(motion_vector_tile_max_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-motion-vector-tile-max-pipeline"),
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
    use super::MOTION_VECTOR_TILE_MAX_SHADER;

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
    fn motion_vector_tile_max_shader_parses_and_selects_dominant_tile_vector() {
        validate_shader_source("motion_vector_tile_max.wgsl", MOTION_VECTOR_TILE_MAX_SHADER);
        assert!(MOTION_VECTOR_TILE_MAX_SHADER
            .contains("@group(0) @binding(0) var motion_vector_source_tex"));
        assert!(MOTION_VECTOR_TILE_MAX_SHADER.contains("textureDimensions"));
        assert!(MOTION_VECTOR_TILE_MAX_SHADER.contains("fn choose_motion_vector_tile_max"));
        assert!(MOTION_VECTOR_TILE_MAX_SHADER.contains("fn motion_vector_tile_max"));
        assert!(MOTION_VECTOR_TILE_MAX_SHADER.contains("textureLoad(motion_vector_source_tex"));
        assert!(MOTION_VECTOR_TILE_MAX_SHADER.contains("tile_coord * vec2<u32>(2u, 2u)"));
    }
}
