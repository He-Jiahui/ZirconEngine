pub(super) fn exposure_histogram_pipeline(
    device: &wgpu::Device,
    exposure_histogram_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-exposure-histogram-shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../../../shaders/exposure_histogram.wgsl").into(),
        ),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-exposure-histogram-pipeline-layout"),
        bind_group_layouts: &[Some(exposure_histogram_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("zircon-exposure-histogram-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    const EXPOSURE_HISTOGRAM_SHADER: &str =
        include_str!("../../../shaders/exposure_histogram.wgsl");

    #[test]
    fn exposure_histogram_shader_declares_compute_entry_and_atomic_bins() {
        assert!(EXPOSURE_HISTOGRAM_SHADER.contains("@compute @workgroup_size(16, 16, 1)"));
        assert!(EXPOSURE_HISTOGRAM_SHADER.contains("fn cs_main"));
        assert!(EXPOSURE_HISTOGRAM_SHADER.contains("array<atomic<u32>, 64>"));
        assert!(EXPOSURE_HISTOGRAM_SHADER.contains("atomicAdd(&exposure_histogram"));
    }
}
