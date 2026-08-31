use zircon_runtime::graphics::{
    RenderPassBufferUploadSink, RenderPassExecutionContext, RenderPassGpuNativeContext,
    RenderPassGpuResourceFactory,
};
use zircon_runtime::render_graph::RenderGraphResourceAccessKind;

use crate::{
    HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS, HYBRID_GI_SCENE_DEPTH_HANDOFF_PIPELINE_LABEL,
    HYBRID_GI_SCENE_DEPTH_HANDOFF_WORKGROUP_SIZE,
};

use super::scene_hzb_camera_packet::{SCENE_HZB_CAMERA_WORD_OFFSET, scene_hzb_camera_packet};
use super::scene_trace_input_packet::{SCENE_TRACE_INPUT_WORD_OFFSET, scene_trace_input_packet};
use super::{
    HYBRID_GI_SCENE_RESOURCE, SCENE_DEPTH_RESOURCE, SCENE_HZB_RESOURCE, SCENE_NORMAL_RESOURCE,
};

enum SceneDepthHandoffShader {
    SingleSample,
    Multisampled,
}

impl SceneDepthHandoffShader {
    fn for_sample_count(sample_count: u32) -> Self {
        if sample_count > 1 {
            Self::Multisampled
        } else {
            Self::SingleSample
        }
    }

    fn label_suffix(&self) -> &'static str {
        match self {
            Self::SingleSample => "single-sample",
            Self::Multisampled => "msaa",
        }
    }

    fn source(&self) -> &'static str {
        match self {
            Self::SingleSample => {
                include_str!("../hybrid_gi/renderer/shaders/scene_depth_handoff.wgsl")
            }
            Self::Multisampled => {
                include_str!("../hybrid_gi/renderer/shaders/scene_depth_handoff_msaa.wgsl")
            }
        }
    }

    fn texture_multisampled(&self) -> bool {
        matches!(self, Self::Multisampled)
    }
}

pub(super) fn record_scene_depth_handoff(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let executor_id = context.executor_id.as_str().to_string();
    let gpu = context.require_gpu()?;
    let scene_depth_desc =
        gpu.require_texture_desc(SCENE_DEPTH_RESOURCE, RenderGraphResourceAccessKind::Read)?;
    let shader = SceneDepthHandoffShader::for_sample_count(scene_depth_desc.sample_count);
    let scene_depth_view = gpu
        .require_texture_view(SCENE_DEPTH_RESOURCE, RenderGraphResourceAccessKind::Read)?
        .clone();
    let scene_normal_view = gpu
        .require_texture_view(SCENE_NORMAL_RESOURCE, RenderGraphResourceAccessKind::Read)?
        .clone();
    let scene_hzb_view = gpu.require_owned_texture_full_mip_view(
        SCENE_HZB_RESOURCE,
        RenderGraphResourceAccessKind::Read,
    )?;
    let hybrid_gi_scene_buffer = gpu.require_buffer_binding(
        HYBRID_GI_SCENE_RESOURCE,
        RenderGraphResourceAccessKind::Write,
    )?;
    let camera_packet = scene_hzb_camera_packet(gpu.frame_extract(), gpu.viewport_size());
    let scene_trace_packet =
        scene_trace_input_packet(&gpu.plugin_outputs().hybrid_gi.scene_prepare);
    let mut buffer_uploads = gpu.buffer_upload_recorder();
    write_buffer_binding(
        &mut buffer_uploads,
        hybrid_gi_scene_buffer.clone(),
        SCENE_HZB_CAMERA_WORD_OFFSET * std::mem::size_of::<u32>() as u64,
        bytemuck::cast_slice(&camera_packet),
        "scene HZB camera packet",
    )?;
    write_buffer_binding(
        &mut buffer_uploads,
        hybrid_gi_scene_buffer.clone(),
        (SCENE_TRACE_INPUT_WORD_OFFSET * std::mem::size_of::<u32>()) as u64,
        bytemuck::cast_slice(&scene_trace_packet),
        "scene trace input packet",
    )?;
    drop(buffer_uploads);

    let mut native = gpu.native_context();
    encode_scene_depth_handoff(
        &mut native,
        &shader,
        &scene_depth_view,
        &scene_normal_view,
        &scene_hzb_view,
        hybrid_gi_scene_buffer,
    );
    drop(native);
    gpu.record_compute_dispatch(
        pass_name,
        executor_id,
        HYBRID_GI_SCENE_DEPTH_HANDOFF_PIPELINE_LABEL,
        HYBRID_GI_SCENE_DEPTH_HANDOFF_WORKGROUP_SIZE,
        HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS,
        vec![HYBRID_GI_SCENE_RESOURCE.to_string()],
    );
    Ok(())
}

fn encode_scene_depth_handoff(
    native: &mut RenderPassGpuNativeContext<'_, '_>,
    shader: &SceneDepthHandoffShader,
    scene_depth_view: &wgpu::TextureView,
    scene_normal_view: &wgpu::TextureView,
    scene_hzb_view: &wgpu::TextureView,
    hybrid_gi_scene_buffer: wgpu::BufferBinding<'_>,
) {
    let bind_group_layout = create_scene_depth_handoff_bind_group_layout(native, shader);
    let pipeline = create_scene_depth_handoff_pipeline(native, shader, &bind_group_layout);
    let bind_group = create_scene_depth_handoff_bind_group(
        native,
        &bind_group_layout,
        scene_depth_view,
        scene_hzb_view,
        hybrid_gi_scene_buffer,
        scene_normal_view,
    );
    let mut pass = native
        .encoder
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("HybridGiSceneDepthHandoffPass"),
            timestamp_writes: None,
        });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(
        HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS[0],
        HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS[1],
        HYBRID_GI_SCENE_DEPTH_HANDOFF_DISPATCH_GROUPS[2],
    );
}

fn create_scene_depth_handoff_bind_group_layout(
    factory: &impl RenderPassGpuResourceFactory,
    shader: &SceneDepthHandoffShader,
) -> wgpu::BindGroupLayout {
    factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-hybrid-gi-scene-depth-handoff-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: shader.texture_multisampled(),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: shader.texture_multisampled(),
                },
                count: None,
            },
        ],
    })
}

fn create_scene_depth_handoff_pipeline(
    factory: &impl RenderPassGpuResourceFactory,
    shader: &SceneDepthHandoffShader,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader_label = format!(
        "zircon-hybrid-gi-scene-depth-handoff-{}-shader",
        shader.label_suffix()
    );
    let shader_module = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(shader_label.as_str()),
        source: wgpu::ShaderSource::Wgsl(shader.source().into()),
    });
    let pipeline_layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-hybrid-gi-scene-depth-handoff-pipeline-layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(HYBRID_GI_SCENE_DEPTH_HANDOFF_PIPELINE_LABEL),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn create_scene_depth_handoff_bind_group(
    factory: &impl RenderPassGpuResourceFactory,
    bind_group_layout: &wgpu::BindGroupLayout,
    scene_depth_view: &wgpu::TextureView,
    scene_hzb_view: &wgpu::TextureView,
    hybrid_gi_scene_buffer: wgpu::BufferBinding<'_>,
    scene_normal_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    factory.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hybrid-gi-scene-depth-handoff-bind-group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(scene_depth_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(scene_hzb_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(hybrid_gi_scene_buffer),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(scene_normal_view),
            },
        ],
    })
}

fn write_buffer_binding(
    buffer_uploads: &mut dyn RenderPassBufferUploadSink,
    binding: wgpu::BufferBinding<'_>,
    relative_offset: u64,
    bytes: &[u8],
    label: &str,
) -> Result<(), String> {
    let byte_count = u64::try_from(bytes.len())
        .map_err(|_| format!("hybrid GI {label} payload length does not fit u64"))?;
    let relative_end = relative_offset
        .checked_add(byte_count)
        .ok_or_else(|| format!("hybrid GI {label} offset overflows its compiler buffer window"))?;
    let backing_remaining = binding
        .buffer
        .size()
        .checked_sub(binding.offset)
        .ok_or_else(|| {
            format!("hybrid GI {label} binding offset exceeds its backing buffer size")
        })?;
    let window_size = binding.size.map_or(backing_remaining, |size| size.get());
    if relative_end > window_size {
        return Err(format!(
            "hybrid GI {label} range [{relative_offset}..{relative_end}) exceeds compiler buffer window size {window_size}"
        ));
    }
    let absolute_offset = binding.offset.checked_add(relative_offset).ok_or_else(|| {
        format!("hybrid GI {label} absolute buffer offset overflows its backing buffer")
    })?;
    buffer_uploads.write_buffer(binding.buffer, absolute_offset, bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    const HANDOFF_MAGIC: u32 = 0x48474944;
    const DEPTH_Q24_SCALE: f32 = 16777215.0;

    #[test]
    fn scene_depth_handoff_records_buffer_writes_without_native_queue_authority() {
        let source = include_str!("scene_depth_handoff.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("scene-depth handoff production source");

        assert!(production.contains("buffer_upload_recorder()"));
        assert!(production.contains("RenderPassBufferUploadSink"));
        assert!(!production.contains("gpu.queue"));
        assert!(!production.contains("queue.write_buffer"));
    }

    #[test]
    fn scene_depth_handoff_msaa_shader_resolves_depth_sample_count() {
        let Some((device, queue)) = test_device() else {
            return;
        };
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hybrid-gi-scene-depth-handoff-msaa-test-depth"),
            size: wgpu::Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
        let normal = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hybrid-gi-scene-depth-handoff-msaa-test-normal"),
            size: wgpu::Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let normal_view = normal.create_view(&wgpu::TextureViewDescriptor::default());
        let (_hzb, hzb_view) = test_hzb_range_texture(&device, &queue);
        const STORAGE_WORD_COUNT: usize = 294;
        let storage = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hybrid-gi-scene-depth-handoff-msaa-test-storage"),
            size: STORAGE_WORD_COUNT as u64 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hybrid-gi-scene-depth-handoff-msaa-test-readback"),
            size: STORAGE_WORD_COUNT as u64 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("hybrid-gi-scene-depth-handoff-msaa-test"),
        });
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hybrid-gi-scene-depth-handoff-msaa-test-clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &normal_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.5,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.25),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }

        let shader = SceneDepthHandoffShader::for_sample_count(4);
        let bind_group_layout = create_scene_depth_handoff_bind_group_layout(&device, &shader);
        let pipeline = create_scene_depth_handoff_pipeline(&device, &shader, &bind_group_layout);
        let bind_group = create_scene_depth_handoff_bind_group(
            &device,
            &bind_group_layout,
            &depth_view,
            &hzb_view,
            storage.as_entire_buffer_binding(),
            &normal_view,
        );
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("hybrid-gi-scene-depth-handoff-msaa-test-compute"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        encoder.copy_buffer_to_buffer(
            &storage,
            0,
            &readback,
            0,
            STORAGE_WORD_COUNT as u64 * std::mem::size_of::<u32>() as u64,
        );
        queue.submit([encoder.finish()]);

        let words = read_u32_words(&device, &readback, STORAGE_WORD_COUNT);
        assert_eq!(words[0], HANDOFF_MAGIC);
        assert_eq!(words[1], 8);
        assert_eq!(words[2], 8);
        assert_eq!(words[3], ((0.25 * DEPTH_Q24_SCALE) + 0.5) as u32);
        assert_eq!(words[4], 4);
        assert_eq!(words[5..8], [4, 4, 3]);
        assert_eq!(words[8], ((0.75 * DEPTH_Q24_SCALE) + 0.5) as u32);
        assert_eq!(words[9], ((0.25 * DEPTH_Q24_SCALE) + 0.5) as u32);
        assert_eq!(words[14..16], [8, 64]);
        assert_eq!(words[16], ((0.25 * DEPTH_Q24_SCALE) + 0.5) as u32);
        let normal_code = (words[19] >> 8) & 63;
        assert!((3..=4).contains(&(normal_code & 7)));
        assert!((3..=4).contains(&((normal_code >> 3) & 7)));
    }

    #[test]
    fn scene_depth_handoff_shaders_emit_hzb_tiles_and_camera_packet_contract() {
        for source in [
            include_str!("../hybrid_gi/renderer/shaders/scene_depth_handoff.wgsl"),
            include_str!("../hybrid_gi/renderer/shaders/scene_depth_handoff_msaa.wgsl"),
        ] {
            assert!(source.contains("textureNumLevels(scene_hzb_tex)"));
            assert!(source.contains("SCENE_HZB_TILE_WORD_OFFSET"));
            assert!(source.contains("SCENE_HZB_TILE_GRID_EXTENT"));
            assert!(source.contains("center_furthest_depth"));
            assert!(source.contains("center_closest_depth"));
            assert!(source.contains("pack_octahedral_normal_6bit"));
            assert!(source.contains("SCENE_NORMAL_CODE_SHIFT"));
        }
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
            label: Some("zircon-hybrid-gi-scene-depth-handoff-msaa-test-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }))
        .ok()
    }

    fn test_hzb_range_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hybrid-gi-scene-depth-handoff-test-hzb"),
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 3,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        for (mip_level, extent) in [(0_u32, 4_u32), (1, 2), (2, 1)] {
            let pixels = vec![[0.75_f32, 0.25, 0.5, 1.0]; (extent * extent) as usize];
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&pixels),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(extent * 16),
                    rows_per_image: Some(extent),
                },
                wgpu::Extent3d {
                    width: extent,
                    height: extent,
                    depth_or_array_layers: 1,
                },
            );
        }
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("hybrid-gi-scene-depth-handoff-test-hzb-view"),
            base_mip_level: 0,
            mip_level_count: Some(3),
            ..Default::default()
        });
        (texture, view)
    }

    fn read_u32_words(device: &wgpu::Device, buffer: &wgpu::Buffer, word_count: usize) -> Vec<u32> {
        let slice = buffer.slice(..(word_count * std::mem::size_of::<u32>()) as u64);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).ok();
        });
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("device poll should complete readback mapping");
        receiver
            .recv()
            .expect("readback mapping callback should run")
            .expect("readback mapping should succeed");
        let mapped = slice.get_mapped_range();
        let words = bytemuck::cast_slice(&mapped[..]).to_vec();
        drop(mapped);
        buffer.unmap();
        words
    }
}
