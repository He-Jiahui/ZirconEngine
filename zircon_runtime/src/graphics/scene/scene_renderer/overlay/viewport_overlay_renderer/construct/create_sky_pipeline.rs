use super::super::super::super::core::DEPTH_FORMAT;

const SKY_SHADER: &str = include_str!("../../../environment/shaders/skybox_procedural.wgsl");

pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) fn create_sky_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    scene_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let sky_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-sky-layout"),
        bind_group_layouts: &[Some(scene_layout)],
        immediate_size: 0,
    });
    let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-sky-shader"),
        source: wgpu::ShaderSource::Wgsl(SKY_SHADER.into()),
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
            depth_compare: Some(wgpu::CompareFunction::Always),
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
mod tests {
    use super::SKY_SHADER;

    #[test]
    fn skybox_shader_reconstructs_camera_ray_before_source_cubemap_sampling() {
        for expected in [
            "fn skybox_world_direction_from_ndc",
            "scene.inverse_view_proj",
            "camera_forward + ndc.x * camera_right + ndc.y * camera_up",
            "let direction = skybox_world_direction_from_ndc(ndc);",
            "fn skybox_fix_cube_lookup",
            "skybox_fix_cube_lookup(rotated, 0.0)",
        ] {
            assert!(
                SKY_SHADER.contains(expected),
                "skybox shader should use `{expected}` for camera-space cubemap lookup"
            );
        }
    }

    #[test]
    fn skybox_shader_is_valid_wgsl() {
        let module = naga::front::wgsl::parse_str(SKY_SHADER)
            .unwrap_or_else(|error| panic!("{}", error.emit_to_string(SKY_SHADER)));
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );

        validator
            .validate(&module)
            .expect("skybox shader should validate");
    }
}
