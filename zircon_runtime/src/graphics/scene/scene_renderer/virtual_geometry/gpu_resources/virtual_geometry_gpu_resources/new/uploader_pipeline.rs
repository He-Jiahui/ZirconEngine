pub(super) fn create_uploader_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-vg-uploader-shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../../shaders/uploader.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-vg-uploader-pipeline-layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("zircon-vg-uploader-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}
