use super::super::super::super::core::DEPTH_FORMAT;

const SKY_SHADER: &str = concat!(
    include_str!("../../../../../shader/wgsl/zr_volumetric.wgsl"),
    "\n",
    include_str!("../../../../../shader/wgsl/zr_procedural_sky.wgsl"),
    "\n",
    include_str!("../../../environment/shaders/skybox_procedural.wgsl"),
);

const SKY_SHADER_BODY: &str = concat!(
    include_str!("../../../../../shader/wgsl/zr_procedural_sky.wgsl"),
    "\n",
    include_str!("../../../environment/shaders/skybox_procedural.wgsl"),
);
const SKY_VOLUMETRIC_DISABLED: &str = r#"
fn zr_volumetric_apply(color: vec3<f32>, _fragment_position: vec2<f32>, _device_depth: f32) -> vec3<f32> {
    return color;
}
"#;

// 禁用体积光时用原样返回的应用函数维持天空入口和统一布局；此着色器变体不声明体积光资源绑定。
fn sky_shader_source(volumetric_enabled: bool) -> String {
    let volumetric = if volumetric_enabled {
        include_str!("../../../../../shader/wgsl/zr_volumetric.wgsl")
    } else {
        SKY_VOLUMETRIC_DISABLED
    };
    format!("{volumetric}\n{SKY_SHADER_BODY}")
}

pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) fn create_sky_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    scene_layout: &wgpu::BindGroupLayout,
    volumetric_layout: &wgpu::BindGroupLayout,
    volumetric_enabled: bool,
) -> wgpu::RenderPipeline {
    let sky_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-sky-layout"),
        bind_group_layouts: &[Some(scene_layout), Some(volumetric_layout)],
        immediate_size: 0,
    });
    let sky_shader_source = sky_shader_source(volumetric_enabled);
    let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-sky-shader"),
        source: wgpu::ShaderSource::Wgsl(sky_shader_source.into()),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-sky-pipeline"),
        layout: Some(&sky_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &sky_shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &sky_shader,
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
#[path = "tests/create_sky_pipeline.rs"]
mod tests;
