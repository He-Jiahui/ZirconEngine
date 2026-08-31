use super::color_space::{target_color_mode, UiTargetColorMode};
use super::geometry::{ImageVertex, SolidInstance, SolidVertex};

const UI_MATERIAL_SHADER: &str = include_str!("shaders/ui_material.wgsl");
const UI_SURFACE_BLEND: wgpu::BlendState = wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FragmentEntryPoints {
    solid: &'static str,
    solid_instance: &'static str,
    image: &'static str,
}

fn fragment_entry_points(target_format: wgpu::TextureFormat) -> FragmentEntryPoints {
    match target_color_mode(target_format) {
        UiTargetColorMode::LinearSrgb => FragmentEntryPoints {
            solid: "solid_fs_linear_target",
            solid_instance: "solid_instance_fs_linear_target",
            image: "image_fs_linear_target",
        },
        UiTargetColorMode::ByteEncodedFallback => FragmentEntryPoints {
            solid: "solid_fs_byte_target",
            solid_instance: "solid_instance_fs_byte_target",
            image: "image_fs_byte_target",
        },
    }
}

pub(super) fn create_damage_clear_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-ui-damage-clear-pipeline-layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-ui-damage-clear-shader"),
        source: wgpu::ShaderSource::Wgsl(UI_MATERIAL_SHADER.into()),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-ui-damage-clear-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("damage_clear_vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("damage_clear_fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

pub(super) fn create_solid_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let entry_points = fragment_entry_points(target_format);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-ui-solid-pipeline-layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-ui-solid-shader"),
        source: wgpu::ShaderSource::Wgsl(UI_MATERIAL_SHADER.into()),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-ui-solid-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("solid_vs_main"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<SolidVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: std::mem::offset_of!(SolidVertex, position) as wgpu::BufferAddress,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: std::mem::offset_of!(SolidVertex, color) as wgpu::BufferAddress,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                    wgpu::VertexAttribute {
                        offset: std::mem::offset_of!(SolidVertex, local_position)
                            as wgpu::BufferAddress,
                        shader_location: 2,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: std::mem::offset_of!(SolidVertex, half_extent)
                            as wgpu::BufferAddress,
                        shader_location: 3,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: std::mem::offset_of!(SolidVertex, corner_radius)
                            as wgpu::BufferAddress,
                        shader_location: 4,
                        format: wgpu::VertexFormat::Float32,
                    },
                    wgpu::VertexAttribute {
                        offset: std::mem::offset_of!(SolidVertex, border_width)
                            as wgpu::BufferAddress,
                        shader_location: 5,
                        format: wgpu::VertexFormat::Float32,
                    },
                    wgpu::VertexAttribute {
                        offset: std::mem::offset_of!(SolidVertex, fill_color)
                            as wgpu::BufferAddress,
                        shader_location: 6,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(entry_points.solid),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(UI_SURFACE_BLEND),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

pub(super) fn create_solid_instance_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let entry_points = fragment_entry_points(target_format);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-ui-solid-instance-pipeline-layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-ui-solid-instance-shader"),
        source: wgpu::ShaderSource::Wgsl(UI_MATERIAL_SHADER.into()),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-ui-solid-instance-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("solid_instance_vs_main"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<SolidInstance>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: std::mem::offset_of!(SolidInstance, min_position)
                            as wgpu::BufferAddress,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: std::mem::offset_of!(SolidInstance, max_position)
                            as wgpu::BufferAddress,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: std::mem::offset_of!(SolidInstance, color) as wgpu::BufferAddress,
                        shader_location: 2,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(entry_points.solid_instance),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(UI_SURFACE_BLEND),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

pub(super) fn create_image_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let entry_points = fragment_entry_points(target_format);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-ui-image-pipeline-layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-ui-image-shader"),
        source: wgpu::ShaderSource::Wgsl(UI_MATERIAL_SHADER.into()),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-ui-image-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("image_vs_main"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<ImageVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some(entry_points.image),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(UI_SURFACE_BLEND),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

pub(super) fn create_image_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-ui-image-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

pub(super) fn create_image_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("zircon-ui-image-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::{fragment_entry_points, UI_MATERIAL_SHADER, UI_SURFACE_BLEND};

    #[test]
    fn ui_material_shader_exposes_surface_entry_points_and_material_helpers() {
        for entry_point in [
            "damage_clear_vs_main",
            "damage_clear_fs_main",
            "solid_vs_main",
            "solid_instance_vs_main",
            "solid_fs_linear_target",
            "solid_fs_byte_target",
            "solid_instance_fs_linear_target",
            "solid_instance_fs_byte_target",
            "image_vs_main",
            "image_fs_linear_target",
            "image_fs_byte_target",
        ] {
            assert!(
                UI_MATERIAL_SHADER.contains(entry_point),
                "ui_material.wgsl must expose `{entry_point}`"
            );
        }

        for helper in [
            "material_tint",
            "premultiply_alpha",
            "srgb_to_linear",
            "linear_to_srgb",
            "rounded_box_distance",
            "rounded_box_alpha",
            "material_solid_color",
            "material_image_color",
        ] {
            assert!(
                UI_MATERIAL_SHADER.contains(helper),
                "ui_material.wgsl must keep the Material UI helper `{helper}`"
            );
        }
    }

    #[test]
    fn fragment_entry_points_follow_the_target_transfer_function() {
        let linear = fragment_entry_points(wgpu::TextureFormat::Bgra8UnormSrgb);
        assert_eq!(linear.solid, "solid_fs_linear_target");
        assert_eq!(linear.solid_instance, "solid_instance_fs_linear_target");
        assert_eq!(linear.image, "image_fs_linear_target");

        let byte = fragment_entry_points(wgpu::TextureFormat::Bgra8Unorm);
        assert_eq!(byte.solid, "solid_fs_byte_target");
        assert_eq!(byte.solid_instance, "solid_instance_fs_byte_target");
        assert_eq!(byte.image, "image_fs_byte_target");
    }

    #[test]
    fn analytic_solid_vertex_abi_carries_pixel_space_shape_parameters() {
        assert_eq!(std::mem::size_of::<super::SolidVertex>(), 64);
        assert_eq!(std::mem::offset_of!(super::SolidVertex, position), 0);
        assert_eq!(std::mem::offset_of!(super::SolidVertex, color), 8);
        assert_eq!(std::mem::offset_of!(super::SolidVertex, local_position), 24);
        assert_eq!(std::mem::offset_of!(super::SolidVertex, half_extent), 32);
        assert_eq!(std::mem::offset_of!(super::SolidVertex, corner_radius), 40);
        assert_eq!(std::mem::offset_of!(super::SolidVertex, border_width), 44);
        assert_eq!(std::mem::offset_of!(super::SolidVertex, fill_color), 48);
        assert!(UI_MATERIAL_SHADER.contains("fwidth(outer_distance)"));
        assert!(UI_MATERIAL_SHADER
            .contains("smoothstep(\n        distance_width * -0.5,\n        distance_width * 0.5,\n        signed_distance,\n    )"));
        assert!(UI_MATERIAL_SHADER.contains("array<vec2<f32>, 16>"));
        assert!(UI_MATERIAL_SHADER.contains("let subpixel_filter_scale = 0.25"));
        assert!(UI_MATERIAL_SHADER.contains("let coverage_guard = distance_width * 0.75"));
        assert!(
            UI_MATERIAL_SHADER.contains("let inner_coverage_guard = inner_distance_width * 0.75")
        );
        assert!(!UI_MATERIAL_SHADER.contains("fwidth(sample_outer_distance)"));
        assert!(UI_MATERIAL_SHADER
            .contains("return vec2<f32>(outer_coverage_sum, inner_coverage_sum) * 0.0625"));
        assert!(UI_MATERIAL_SHADER.contains("coverages.x - coverages.y"));
        assert!(UI_MATERIAL_SHADER.contains("return fill + border"));
    }

    #[test]
    fn compact_solid_instance_abi_is_one_record_per_quad() {
        assert_eq!(std::mem::size_of::<super::SolidInstance>(), 32);
        assert_eq!(std::mem::offset_of!(super::SolidInstance, min_position), 0);
        assert_eq!(std::mem::offset_of!(super::SolidInstance, max_position), 8);
        assert_eq!(std::mem::offset_of!(super::SolidInstance, color), 16);
        assert!(UI_MATERIAL_SHADER.contains("@builtin(vertex_index) vertex_index: u32"));
        assert!(UI_MATERIAL_SHADER.contains("array<vec2<f32>, 6>"));
    }

    #[test]
    fn ui_material_shader_routes_fragment_outputs_through_material_helpers() {
        assert_eq!(
            UI_SURFACE_BLEND,
            wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
            "solid and image UI surfaces must blend premultiplied fragment output"
        );
        assert!(
            UI_MATERIAL_SHADER.contains("return material_solid_color(input.color, true);")
                && UI_MATERIAL_SHADER.contains("return material_solid_color(input.color, false);"),
            "flat solid fragment output must go through the Material solid color path"
        );
        assert!(
            UI_MATERIAL_SHADER.contains("input.color.a * coverage"),
            "analytic rounded solids must apply distance-field coverage before premultiplication"
        );
        assert!(
            UI_MATERIAL_SHADER
                .contains("premultiply_alpha(vec4<f32>(srgb_to_linear(tinted.rgb), tinted.a))"),
            "linear targets must decode solid sRGB before coverage alpha premultiplication"
        );
        assert!(
            UI_MATERIAL_SHADER
                .contains("return material_image_color(textureSample(source_texture, source_sampler, input.uv), true);")
                && UI_MATERIAL_SHADER.contains(
                    "return material_image_color(textureSample(source_texture, source_sampler, input.uv), false);"
                ),
            "image fragment output must go through the Material image color path"
        );
        let image_helper = UI_MATERIAL_SHADER
            .split("fn material_image_color")
            .nth(1)
            .and_then(|source| source.split('}').next())
            .expect("image material helper must remain explicit");
        assert!(
            !image_helper.contains("premultiply_alpha"),
            "filtered image texels are already premultiplied and must not be multiplied twice"
        );
        assert!(
            UI_MATERIAL_SHADER.contains("linear_to_srgb(clamp(tinted.rgb / tinted.a")
                && UI_MATERIAL_SHADER.contains("straight_srgb * tinted.a"),
            "the byte-target fallback must encode straight linear RGB before restoring premultiplication"
        );
    }
}
