pub(super) fn cluster_pipeline(
    device: &wgpu::Device,
    cluster_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-clustered-lighting-shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../../../shaders/clustered_lighting.wgsl").into(),
        ),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-cluster-pipeline-layout"),
        bind_group_layouts: &[Some(cluster_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("zircon-cluster-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}
