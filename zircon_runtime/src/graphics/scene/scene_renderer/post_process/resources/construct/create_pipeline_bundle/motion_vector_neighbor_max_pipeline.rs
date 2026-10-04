//! 创建从粗粒度运动矢量瓦片读取邻域最大值的片元管线，输出固定的 RGBA16Float 运动纹理。
const MOTION_VECTOR_NEIGHBOR_MAX_SHADER: &str =
    include_str!("../../../shaders/motion_vector_neighbor_max.wgsl");
const MOTION_VECTOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// 将邻域最大值着色器绑定到单纹理布局，供运动模糊和后续后处理阶段消费。
pub(super) fn motion_vector_neighbor_max_pipeline(
    device: &wgpu::Device,
    motion_vector_neighbor_max_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-motion-vector-neighbor-max-shader"),
        source: wgpu::ShaderSource::Wgsl(MOTION_VECTOR_NEIGHBOR_MAX_SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-motion-vector-neighbor-max-pipeline-layout"),
        bind_group_layouts: &[Some(motion_vector_neighbor_max_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-motion-vector-neighbor-max-pipeline"),
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
#[path = "tests/motion_vector_neighbor_max_pipeline.rs"]
mod tests;
