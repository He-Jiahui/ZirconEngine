pub(super) fn exposure_resolve_pipeline(
    device: &wgpu::Device,
    exposure_resolve_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-exposure-resolve-shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../../../shaders/exposure_resolve.wgsl").into(),
        ),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-exposure-resolve-pipeline-layout"),
        bind_group_layouts: &[Some(exposure_resolve_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("zircon-exposure-resolve-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    const EXPOSURE_RESOLVE_SHADER: &str = include_str!("../../../shaders/exposure_resolve.wgsl");

    #[test]
    fn exposure_resolve_shader_declares_manual_and_histogram_output() {
        assert!(EXPOSURE_RESOLVE_SHADER.contains("@compute @workgroup_size(1, 1, 1)"));
        assert!(EXPOSURE_RESOLVE_SHADER.contains("fn histogram_average_ev100"));
        assert!(EXPOSURE_RESOLVE_SHADER.contains("fn adapt_ev100"));
        assert!(EXPOSURE_RESOLVE_SHADER.contains("current_exposure[0]"));
    }
}
