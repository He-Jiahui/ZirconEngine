pub(super) fn hzb_pipeline(
    device: &wgpu::Device,
    hzb_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-hzb-build-shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../../shaders/hzb_build.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-hzb-build-pipeline-layout"),
        bind_group_layouts: &[Some(hzb_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("zircon-hzb-build-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    const HZB_BUILD_SHADER: &str = include_str!("../../../shaders/hzb_build.wgsl");

    #[test]
    fn hzb_shader_declares_reduce_entry_and_storage_target() {
        assert!(HZB_BUILD_SHADER.contains("@compute @workgroup_size(8, 8, 1)"));
        assert!(HZB_BUILD_SHADER.contains("fn cs_main"));
        assert!(HZB_BUILD_SHADER.contains("texture_storage_2d<rgba16float, write>"));
    }
}
