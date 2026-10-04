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
#[path = "tests/exposure_resolve_pipeline.rs"]
mod tests;
