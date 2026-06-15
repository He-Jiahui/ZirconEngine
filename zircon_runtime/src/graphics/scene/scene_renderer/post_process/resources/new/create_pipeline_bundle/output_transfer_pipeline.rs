use super::super::super::shader_sources::OUTPUT_TRANSFER_SHADER;

pub(super) fn output_transfer_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    output_transfer_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-output-transfer-shader"),
        source: wgpu::ShaderSource::Wgsl(OUTPUT_TRANSFER_SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-output-transfer-pipeline-layout"),
        bind_group_layouts: &[Some(output_transfer_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-output-transfer-pipeline"),
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
                format: target_format,
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
    use super::super::super::super::shader_sources::OUTPUT_TRANSFER_SHADER;

    #[test]
    fn output_transfer_shader_parses() {
        let module = naga::front::wgsl::parse_str(OUTPUT_TRANSFER_SHADER)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(OUTPUT_TRANSFER_SHADER)));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator
            .validate(&module)
            .unwrap_or_else(|error| panic!("{error}"));
    }
}
