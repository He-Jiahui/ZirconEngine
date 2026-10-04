use crate::graphics::shader::motion_vector_tile_max_pass_plan;

const MOTION_VECTOR_TILE_MAX_FRAGMENT_SHADER: &str =
    include_str!("../../../shaders/motion_vector_tile_max.wgsl");
const FULLSCREEN_TRIANGLE_SHADER: &str =
    include_str!("../../../../../../shader/wgsl/zr_fullscreen_triangle.wgsl");
const MOTION_VECTOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub(super) fn motion_vector_tile_max_pipeline(
    device: &wgpu::Device,
    motion_vector_tile_max_bind_group_layout: &wgpu::BindGroupLayout,
    motion_vector_tile_max_parameter_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let plan = motion_vector_tile_max_pass_plan();
    let shader_source =
        format!("{FULLSCREEN_TRIANGLE_SHADER}\n{MOTION_VECTOR_TILE_MAX_FRAGMENT_SHADER}");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-motion-vector-tile-max-shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-motion-vector-tile-max-pipeline-layout"),
        // 资源和参数分别沿用全屏 ABI 的第一、第二组；保留第零组空位，防止布局压缩后组号偏移。
        bind_group_layouts: &[
            None,
            Some(motion_vector_tile_max_bind_group_layout),
            Some(motion_vector_tile_max_parameter_bind_group_layout),
        ],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&plan.pipeline_label),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some(&plan.vertex_entry),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(&plan.shader.fragment_entry),
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
#[path = "tests/motion_vector_tile_max_pipeline.rs"]
mod tests;
