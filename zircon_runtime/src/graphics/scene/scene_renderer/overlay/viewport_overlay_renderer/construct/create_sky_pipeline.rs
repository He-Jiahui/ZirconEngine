use super::super::super::super::core::DEPTH_FORMAT;

const SKY_SHADER: &str = concat!(
    include_str!("../../../../../shader/wgsl/zr_volumetric.wgsl"),
    "\n",
    include_str!("../../../environment/shaders/skybox_procedural.wgsl"),
);

const SKY_SHADER_BODY: &str = include_str!("../../../environment/shaders/skybox_procedural.wgsl");
const SKY_VOLUMETRIC_DISABLED: &str = r#"
fn zr_volumetric_apply(color: vec3<f32>, _fragment_position: vec2<f32>, _device_depth: f32) -> vec3<f32> {
    return color;
}
"#;

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
mod tests {
    use super::{SKY_SHADER, sky_shader_source};

    #[test]
    fn skybox_shader_variant_removes_volumetric_bindings_when_disabled() {
        let disabled = sky_shader_source(false);
        let enabled = sky_shader_source(true);

        for binding in ["@binding(25)", "@binding(26)", "@binding(27)"] {
            assert!(!disabled.contains(binding));
            assert!(enabled.contains(binding));
        }
    }

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
    fn skybox_shader_rotation_uses_cpu_precomputed_trigonometry() {
        let rotation = SKY_SHADER
            .split("fn skybox_rotated_direction_normalized(")
            .nth(1)
            .and_then(|source| source.split("fn skybox_normalize_or_fallback(").next())
            .expect("skybox shader should retain its rotation helper");

        let environment_sample_params = SKY_SHADER
            .find("environment_sample_params: vec4<f32>,")
            .expect("skybox SceneUniform should retain the environment sampling parameters");
        let rotation_tail = SKY_SHADER
            .find("environment_rotation_sin_cos: vec4<f32>,")
            .expect("skybox SceneUniform should append the rotation tail");
        assert!(
            environment_sample_params < rotation_tail,
            "the skybox SceneUniform mirror must append the rotation tail after existing fields"
        );
        assert!(rotation.contains("scene.environment_rotation_sin_cos.z < 0.5"));
        assert!(
            rotation.contains("scene.environment_rotation_sin_cos.x")
                && rotation.contains("scene.environment_rotation_sin_cos.y")
        );
        assert!(!rotation.contains("sin(rotation)"));
        assert!(!rotation.contains("cos(rotation)"));
    }

    #[test]
    fn skybox_shader_feature_variants_are_valid_wgsl() {
        for (label, source) in [
            ("disabled", sky_shader_source(false)),
            ("enabled", sky_shader_source(true)),
        ] {
            let module = naga::front::wgsl::parse_str(&source)
                .unwrap_or_else(|error| panic!("{label}: {}", error.emit_to_string(&source)));
            let mut validator = naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::all(),
            );

            validator
                .validate(&module)
                .unwrap_or_else(|error| panic!("{label} skybox shader should validate: {error}"));
        }
    }

    #[test]
    fn skybox_shader_applies_integrated_volumetric_lighting_at_far_depth() {
        for expected in [
            "output.clip_position = vec4<f32>(position, 1.0, 1.0);",
            "@group(1) @binding(25) var<uniform> zr_volumetric_apply_params",
            "@group(1) @binding(26) var zr_volumetric_integrated: texture_3d<f32>;",
            "@group(1) @binding(27) var zr_volumetric_sampler: sampler;",
            "zr_volumetric_apply(color, input.clip_position.xy, 1.0)",
        ] {
            assert!(
                SKY_SHADER.contains(expected),
                "skybox shader should use volumetric contract `{expected}`"
            );
        }
    }
}
