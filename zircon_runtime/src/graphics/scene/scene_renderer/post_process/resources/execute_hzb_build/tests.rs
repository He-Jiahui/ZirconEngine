use std::sync::mpsc;

use crate::core::math::UVec2;
use crate::graphics::shader::{
    create_compute_shader_bind_group_layout, hzb_build_dispatch_plan, hzb_build_msaa_dispatch_plan,
    ShaderWgpuResourceDescriptor, HZB_SCENE_DEPTH_RESOURCE, HZB_SOURCE_RESOURCE,
    HZB_TARGET_RESOURCE,
};
use crate::graphics::visibility::HzbBuilder;

use super::execute_hzb_build::{
    create_hzb_params_upload_buffer, execute_hzb_build_mip_with_resources, HzbBuildMipResources,
};

const TEST_SCENE_SIZE: UVec2 = UVec2::new(4, 4);
const F16_ONE_BITS: u16 = 0x3c00;
const COPY_BYTES_PER_ROW: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
const ZR_REDUCE_INCLUDE: &str = include_str!("../../../../../shader/includes/zr_reduce.wgsl");

#[test]
fn hzb_params_upload_uses_fixed_stack_storage() {
    let source = include_str!("execute_hzb_build.rs");

    assert!(
        !source.contains("collect::<Vec<_>>"),
        "per-frame HZB parameter packing should not allocate a temporary Vec"
    );
    assert!(
        source.contains("HZB_MAX_MIP_COUNT"),
        "HZB parameter packing should use the bounded u32 mip domain"
    );
}

const SINGLE_SAMPLE_MIP0_TEXELS: [[u16; 4]; 4] = [
    [0x3200, 0x2800, 0x3100, F16_ONE_BITS],
    [0x3400, 0x2e00, 0x3100, F16_ONE_BITS],
    [0x3700, 0x3480, 0x3100, F16_ONE_BITS],
    [0x3800, 0x3580, 0x3100, F16_ONE_BITS],
];
const SINGLE_SAMPLE_MIP1_TEXELS: [[u16; 4]; 1] = [[0x3800, 0x2800, 0x3780, F16_ONE_BITS]];
const MULTISAMPLE_MIP0_TEXELS: [[u16; 4]; 4] = [
    [0x3380, 0x2800, 0x3280, F16_ONE_BITS],
    [0x34c0, 0x2e00, 0x3280, F16_ONE_BITS],
    [0x37c0, 0x3480, 0x3280, F16_ONE_BITS],
    [0x3860, 0x3580, 0x3280, F16_ONE_BITS],
];
const MULTISAMPLE_MIP1_TEXELS: [[u16; 4]; 1] = [[0x3860, 0x2800, 0x3820, F16_ONE_BITS]];

#[test]
fn hzb_build_preserves_per_mip_params_and_resolves_msaa_depth() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping HZB WGPU regression because no adapter is available");
        return;
    };
    let resources = TestHzbBuildResources::new(&device);

    for sample_count in [1, 4] {
        assert_hzb_depth_chain(&device, &queue, &resources, sample_count);
    }
}

fn assert_hzb_depth_chain(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    resources: &TestHzbBuildResources,
    sample_count: u32,
) {
    let plan = HzbBuilder::new(TEST_SCENE_SIZE).build_plan();
    assert_eq!(plan.hzb_size, UVec2::new(2, 2));
    assert_eq!(plan.mip_count, 2);

    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-hzb-wgpu-regression-scene-depth"),
        size: wgpu::Extent3d {
            width: TEST_SCENE_SIZE.x,
            height: TEST_SCENE_SIZE.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let hzb_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-hzb-wgpu-regression-output"),
        size: wgpu::Extent3d {
            width: plan.hzb_size.x,
            height: plan.hzb_size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: plan.mip_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let readback_size = COPY_BYTES_PER_ROW as u64 * 3;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-hzb-wgpu-regression-readback"),
        size: readback_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hzb-wgpu-regression-encoder"),
    });
    let params_upload_buffer = create_hzb_params_upload_buffer(device, plan);
    encode_depth_pattern(device, &mut encoder, &depth_view, sample_count);

    for mip_level in 0..plan.mip_count {
        let source_view = (mip_level > 0).then(|| hzb_mip_view(&hzb_texture, mip_level - 1));
        let target_view = hzb_mip_view(&hzb_texture, mip_level);
        let (bind_group_layout, pipeline) = resources.pipeline_for_sample_count(sample_count);
        execute_hzb_build_mip_with_resources(
            device,
            &mut encoder,
            &depth_view,
            source_view.as_ref(),
            &target_view,
            plan.mip_size(mip_level),
            mip_level,
            &params_upload_buffer,
            None,
            HzbBuildMipResources {
                bind_group_layout,
                pipeline,
                params_buffer: &resources.params_buffer,
                fallback_source_view: &resources.fallback_source_view,
            },
        );
    }

    let mut readback_offset = 0_u64;
    for mip_level in 0..plan.mip_count {
        let mip_size = plan.mip_size(mip_level);
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &hzb_texture,
                mip_level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: readback_offset,
                    bytes_per_row: Some(COPY_BYTES_PER_ROW),
                    rows_per_image: Some(mip_size.y),
                },
            },
            wgpu::Extent3d {
                width: mip_size.x,
                height: mip_size.y,
                depth_or_array_layers: 1,
            },
        );
        readback_offset += u64::from(COPY_BYTES_PER_ROW * mip_size.y);
    }
    queue.submit([encoder.finish()]);

    let bytes = readback_bytes(device, &readback, readback_size);
    let mut readback_offset = 0_usize;
    for mip_level in 0..plan.mip_count {
        let mip_size = plan.mip_size(mip_level);
        let expected_texels = expected_hzb_texels(sample_count, mip_level);
        assert_eq!(expected_texels.len(), (mip_size.x * mip_size.y) as usize);
        for y in 0..mip_size.y as usize {
            for x in 0..mip_size.x as usize {
                let offset = readback_offset + y * COPY_BYTES_PER_ROW as usize + x * 8;
                let expected = expected_texels[y * mip_size.x as usize + x];
                assert_eq!(
                    [
                        read_u16(&bytes, offset),
                        read_u16(&bytes, offset + 2),
                        read_u16(&bytes, offset + 4),
                        read_u16(&bytes, offset + 6),
                    ],
                    expected,
                    "sample_count={sample_count} mip={mip_level} texel=({x},{y})",
                );
            }
        }
        readback_offset += COPY_BYTES_PER_ROW as usize * mip_size.y as usize;
    }
}

fn expected_hzb_texels(sample_count: u32, mip_level: u32) -> &'static [[u16; 4]] {
    match (sample_count, mip_level) {
        (1, 0) => &SINGLE_SAMPLE_MIP0_TEXELS,
        (1, 1) => &SINGLE_SAMPLE_MIP1_TEXELS,
        (4, 0) => &MULTISAMPLE_MIP0_TEXELS,
        (4, 1) => &MULTISAMPLE_MIP1_TEXELS,
        _ => panic!("unsupported HZB regression expectation"),
    }
}

fn encode_depth_pattern(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    depth_view: &wgpu::TextureView,
    sample_count: u32,
) {
    const DEPTH_PATTERN_SHADER: &str = r#"
        @vertex
        fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
            let positions = array<vec2<f32>, 3>(
                vec2<f32>(-1.0, -3.0),
                vec2<f32>(-1.0, 1.0),
                vec2<f32>(3.0, 1.0),
            );
            return vec4<f32>(positions[vertex_index], 0.0, 1.0);
        }

        @fragment
        fn fs_main(
            @builtin(position) position: vec4<f32>,
            @builtin(sample_index) sample_index: u32,
        ) -> @builtin(frag_depth) f32 {
            let pixel_index = u32(position.x) + u32(position.y) * 4u;
            return f32((pixel_index + 1u) * 2u + sample_index) / 64.0;
        }
    "#;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-hzb-wgpu-regression-depth-pattern-shader"),
        source: wgpu::ShaderSource::Wgsl(DEPTH_PATTERN_SHADER.into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-hzb-wgpu-regression-depth-pattern-pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Always),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[],
        }),
        multiview_mask: None,
        cache: None,
    });

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("zircon-hzb-wgpu-regression-depth-pattern-pass"),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });
    pass.set_pipeline(&pipeline);
    pass.draw(0..3, 0..1);
}

struct TestHzbBuildResources {
    single_sample_layout: wgpu::BindGroupLayout,
    multisample_layout: wgpu::BindGroupLayout,
    single_sample_pipeline: wgpu::ComputePipeline,
    multisample_pipeline: wgpu::ComputePipeline,
    params_buffer: wgpu::Buffer,
    _fallback_source_texture: wgpu::Texture,
    fallback_source_view: wgpu::TextureView,
}

impl TestHzbBuildResources {
    fn new(device: &wgpu::Device) -> Self {
        let single_sample_layout = hzb_bind_group_layout(device, false);
        let multisample_layout = hzb_bind_group_layout(device, true);
        let single_sample_pipeline = hzb_pipeline(
            device,
            &single_sample_layout,
            include_str!("../../shaders/hzb_build.wgsl"),
            false,
        );
        let multisample_pipeline = hzb_pipeline(
            device,
            &multisample_layout,
            include_str!("../../shaders/hzb_build_msaa.wgsl"),
            true,
        );
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-hzb-wgpu-regression-params"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let fallback_source_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-hzb-wgpu-regression-fallback-source"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let fallback_source_view =
            fallback_source_texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            single_sample_layout,
            multisample_layout,
            single_sample_pipeline,
            multisample_pipeline,
            params_buffer,
            _fallback_source_texture: fallback_source_texture,
            fallback_source_view,
        }
    }

    fn pipeline_for_sample_count(
        &self,
        sample_count: u32,
    ) -> (&wgpu::BindGroupLayout, &wgpu::ComputePipeline) {
        if sample_count > 1 {
            (&self.multisample_layout, &self.multisample_pipeline)
        } else {
            (&self.single_sample_layout, &self.single_sample_pipeline)
        }
    }
}

fn hzb_bind_group_layout(device: &wgpu::Device, multisampled: bool) -> wgpu::BindGroupLayout {
    let plan = if multisampled {
        hzb_build_msaa_dispatch_plan()
    } else {
        hzb_build_dispatch_plan()
    };
    create_compute_shader_bind_group_layout(
        device,
        plan,
        &[
            ShaderWgpuResourceDescriptor::texture(
                HZB_SCENE_DEPTH_RESOURCE,
                wgpu::TextureSampleType::Depth,
                wgpu::TextureViewDimension::D2,
                multisampled,
            ),
            ShaderWgpuResourceDescriptor::texture(
                HZB_SOURCE_RESOURCE,
                wgpu::TextureSampleType::Float { filterable: false },
                wgpu::TextureViewDimension::D2,
                false,
            ),
            ShaderWgpuResourceDescriptor::storage_texture(
                HZB_TARGET_RESOURCE,
                wgpu::TextureFormat::Rgba16Float,
                wgpu::TextureViewDimension::D2,
            ),
        ],
    )
    .expect("HZB test layout must match the production compute contract")
}

fn hzb_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    shader_source: &str,
    multisampled: bool,
) -> wgpu::ComputePipeline {
    let plan = if multisampled {
        hzb_build_msaa_dispatch_plan()
    } else {
        hzb_build_dispatch_plan()
    };
    let shader_source = [ZR_REDUCE_INCLUDE, shader_source].concat();
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&plan.pipeline_label),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&plan.pipeline_label),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(&plan.pipeline_label),
        layout: Some(&layout),
        module: &shader,
        entry_point: Some(&plan.kernel.kernel),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn hzb_mip_view(texture: &wgpu::Texture, mip_level: u32) -> wgpu::TextureView {
    texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("zircon-hzb-wgpu-regression-mip-view"),
        base_mip_level: mip_level,
        mip_level_count: Some(1),
        ..Default::default()
    })
}

fn readback_bytes(device: &wgpu::Device, buffer: &wgpu::Buffer, size: u64) -> Vec<u8> {
    let slice = buffer.slice(..size);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).ok();
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receiver.recv().unwrap().unwrap();
    let mapped = slice.get_mapped_range();
    let bytes = mapped.to_vec();
    drop(mapped);
    buffer.unmap();
    bytes
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::PRIMARY;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-hzb-wgpu-regression-device"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
}
