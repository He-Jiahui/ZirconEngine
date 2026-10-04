//! 创建颜色 LUT 烘焙计算管线；着色器写入三维 rgba16float 存储纹理。
/// 将 LUT 烘焙着色器与其绑定布局组合为一次初始化的计算管线。
pub(super) fn color_lut_bake_pipeline(
    device: &wgpu::Device,
    color_lut_bake_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-color-lut-bake-shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../../../shaders/color_lut_bake.wgsl").into(),
        ),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-color-lut-bake-pipeline-layout"),
        bind_group_layouts: &[Some(color_lut_bake_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("zircon-color-lut-bake-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

#[cfg(test)]
#[path = "tests/color_lut_bake_pipeline.rs"]
mod tests;
