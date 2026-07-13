use zircon_runtime::graphics::{RenderPassExecutionContext, RenderPassGpuExecutionContext};
use zircon_runtime::render_graph::RenderGraphResourceAccessKind;

use crate::{
    HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS, HYBRID_GI_TRACE_SCHEDULE_PIPELINE_LABEL,
    HYBRID_GI_TRACE_SCHEDULE_WORKGROUP_SIZE,
};

use super::{HYBRID_GI_SCENE_RESOURCE, HYBRID_GI_TRACE_RESOURCE, SCENE_HZB_RESOURCE};

pub(super) fn record_trace_schedule_handoff(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    let pass_name = context.pass_name.clone();
    let executor_id = context.executor_id.as_str().to_string();
    let gpu = context.require_gpu()?;
    let hybrid_gi_scene_buffer = gpu
        .require_buffer(
            HYBRID_GI_SCENE_RESOURCE,
            RenderGraphResourceAccessKind::Read,
        )?
        .clone();
    let hybrid_gi_trace_buffer = gpu
        .require_buffer(
            HYBRID_GI_TRACE_RESOURCE,
            RenderGraphResourceAccessKind::Write,
        )?
        .clone();
    let scene_hzb_view = gpu.require_owned_texture_full_mip_view(
        SCENE_HZB_RESOURCE,
        RenderGraphResourceAccessKind::Read,
    )?;
    encode_trace_schedule_handoff(
        gpu,
        &hybrid_gi_scene_buffer,
        &hybrid_gi_trace_buffer,
        &scene_hzb_view,
    );
    gpu.record_compute_dispatch(
        pass_name,
        executor_id,
        HYBRID_GI_TRACE_SCHEDULE_PIPELINE_LABEL,
        HYBRID_GI_TRACE_SCHEDULE_WORKGROUP_SIZE,
        HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS,
        vec![HYBRID_GI_TRACE_RESOURCE.to_string()],
    );
    Ok(())
}

fn encode_trace_schedule_handoff(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    hybrid_gi_scene_buffer: &wgpu::Buffer,
    hybrid_gi_trace_buffer: &wgpu::Buffer,
    scene_hzb_view: &wgpu::TextureView,
) {
    let bind_group_layout = create_trace_schedule_handoff_bind_group_layout(gpu.device);
    let pipeline = create_trace_schedule_handoff_pipeline(gpu.device, &bind_group_layout);
    let bind_group = create_trace_schedule_handoff_bind_group(
        gpu.device,
        &bind_group_layout,
        hybrid_gi_scene_buffer,
        hybrid_gi_trace_buffer,
        scene_hzb_view,
    );
    let mut pass = gpu
        .encoder
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("HybridGiTraceScheduleHandoffPass"),
            timestamp_writes: None,
        });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(
        HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS[0],
        HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS[1],
        HYBRID_GI_TRACE_SCHEDULE_DISPATCH_GROUPS[2],
    );
}

fn create_trace_schedule_handoff_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-hybrid-gi-trace-schedule-handoff-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

fn create_trace_schedule_handoff_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-hybrid-gi-trace-schedule-handoff-shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../hybrid_gi/renderer/shaders/trace_schedule_handoff.wgsl").into(),
        ),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-hybrid-gi-trace-schedule-handoff-pipeline-layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(HYBRID_GI_TRACE_SCHEDULE_PIPELINE_LABEL),
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn create_trace_schedule_handoff_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    hybrid_gi_scene_buffer: &wgpu::Buffer,
    hybrid_gi_trace_buffer: &wgpu::Buffer,
    scene_hzb_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hybrid-gi-trace-schedule-handoff-bind-group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: hybrid_gi_scene_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: hybrid_gi_trace_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(scene_hzb_view),
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use wgpu::util::DeviceExt;

    use super::*;

    const SCENE_DEPTH_HANDOFF_MAGIC: u32 = 0x4847_4944;
    const TRACE_SCHEDULE_MAGIC: u32 = 0x4847_4954;
    const HZB_TRACE_MAGIC: u32 = 0x4847_5a42;
    const CAMERA_PACKET_MAGIC: u32 = 0x4847_4943;
    const SCENE_TRACE_INPUT_MAGIC: u32 = 0x4847_4949;
    const SURFACE_RADIANCE_RGBA8: u32 = 0xff60_3018;
    const VOXEL_RADIANCE_RGBA8: u32 = 0xff48_240c;
    const DEPTH_Q24_MAX: u32 = 16_777_215;
    const SCENE_WORD_COUNT: usize = 710;
    const TRACE_WORD_COUNT: usize = 576;
    const DEFAULT_NORMAL_CODE: u32 = 36;

    #[test]
    fn trace_schedule_shader_marches_main_scene_hzb_and_samples_surface_cache_radiance() {
        let Some((device, queue)) = test_device() else {
            return;
        };
        let start_depth_q24 = quantize_depth_q24(0.25);
        let target_depth_q24 = quantize_depth_q24(0.75);
        let hit_depth = 0.25 + (0.75 - 0.25) * (4.0 / 7.0);
        let hit_depth_q24 = quantize_depth_q24(hit_depth);
        let mut scene_words = test_scene_words(start_depth_q24, target_depth_q24);
        scene_words[295] = 1;
        scene_words[302..310].copy_from_slice(&[
            7,
            11,
            13,
            SURFACE_RADIANCE_RGBA8,
            0.125_f32.to_bits(),
            0.875_f32.to_bits(),
            hit_depth.to_bits(),
            0.25_f32.to_bits(),
        ]);
        let (_hzb_texture, hzb_view) = test_main_scene_hzb(&device, &queue, hit_depth);
        let words = run_trace_schedule_shader(&device, &queue, &scene_words, &hzb_view);
        assert_eq!(words[0], TRACE_SCHEDULE_MAGIC);
        assert_eq!(words[1], 64);
        assert_eq!(words[2..4], [16, 16]);
        assert_eq!(words[4], target_depth_q24);
        assert_eq!(words[5], 1);
        assert_eq!(words[6], SURFACE_RADIANCE_RGBA8);
        assert_eq!(words[7], 1);
        assert_eq!(words[9], SCENE_DEPTH_HANDOFF_MAGIC);
        assert_eq!(words[10], HZB_TRACE_MAGIC);
        assert_eq!(words[11..14], [8, 8, 4]);
        assert_eq!(words[19..22], [1, 8, 64]);
        assert_eq!(words[22], CAMERA_PACKET_MAGIC);
        assert_eq!(words[44], 64);
        assert_eq!(words[45..49], [8, 1, 0, 0]);
        assert_eq!(words[49], 0x1234_5678);
        assert_eq!(words[64], SURFACE_RADIANCE_RGBA8);
        assert_eq!(words[65], hit_depth_q24);
        assert_ne!(
            words[66], 0,
            "first HZB tile should reconstruct world distance"
        );
        assert_ne!(words[67] & (1 << 8), 0, "first tile should hit scene HZB");
        assert_ne!(
            words[67] & (1 << 10),
            0,
            "first tile should sample surface cache"
        );
        assert_ne!(words[67] & (1 << 12), 0, "first tile should carry radiance");
        assert_eq!(
            words[67] >> 24,
            1,
            "first tile should perform one coarse skip"
        );
        assert_eq!(words[68], 4, "first tile should hit tile coordinate (4, 0)");
        assert_ne!(
            words[70], 0,
            "surface-cache trace tile should carry a local support signature"
        );
        assert_eq!(words[71], DEFAULT_NORMAL_CODE);

        let mut changed_radiance_scene_words = scene_words.clone();
        changed_radiance_scene_words[305] = 0xff18_3060;
        let changed_radiance_words =
            run_trace_schedule_shader(&device, &queue, &changed_radiance_scene_words, &hzb_view);
        assert_ne!(
            changed_radiance_words[70] & 1023,
            words[70] & 1023,
            "surface page radiance changes must invalidate the local temporal support signature"
        );

        let mut changed_bounds_scene_words = scene_words;
        changed_bounds_scene_words[309] = 0.3_f32.to_bits();
        let changed_bounds_words =
            run_trace_schedule_shader(&device, &queue, &changed_bounds_scene_words, &hzb_view);
        assert_ne!(
            changed_bounds_words[70] & 1023,
            words[70] & 1023,
            "surface page topology changes must invalidate the local temporal support signature"
        );
    }

    #[test]
    fn trace_schedule_shader_falls_back_to_world_space_voxel_clipmap_radiance() {
        let Some((device, queue)) = test_device() else {
            return;
        };
        let start_depth_q24 = quantize_depth_q24(0.25);
        let target_depth_q24 = quantize_depth_q24(0.75);
        let hit_depth = 0.25 + (0.75 - 0.25) * (4.0 / 7.0);
        let mut scene_words = test_scene_words(start_depth_q24, target_depth_q24);
        scene_words[296] = 1;
        scene_words[297] = 1;
        scene_words[430..436].copy_from_slice(&[
            5,
            0.0_f32.to_bits(),
            0.0_f32.to_bits(),
            0.0_f32.to_bits(),
            1.0_f32.to_bits(),
            4,
        ]);
        scene_words[454..458].copy_from_slice(&[5, 62, VOXEL_RADIANCE_RGBA8, 1]);
        let (_hzb_texture, hzb_view) = test_main_scene_hzb(&device, &queue, hit_depth);

        let words = run_trace_schedule_shader(&device, &queue, &scene_words, &hzb_view);

        assert_eq!(words[45..49], [8, 0, 1, 1]);
        assert_eq!(words[6], VOXEL_RADIANCE_RGBA8);
        assert_eq!(words[64], VOXEL_RADIANCE_RGBA8);
        assert_ne!(words[67] & (1 << 8), 0, "first tile should hit scene HZB");
        assert_eq!(
            words[67] & (1 << 10),
            0,
            "surface-cache flag should remain clear without a resident page"
        );
        assert_ne!(
            words[67] & (1 << 11),
            0,
            "first tile should use voxel clipmap fallback"
        );
        assert_ne!(
            words[67] & (1 << 12),
            0,
            "voxel fallback should carry radiance"
        );
        assert_ne!(
            words[70], 0,
            "voxel trace tile should carry a local support signature"
        );

        let mut changed_occupancy_scene_words = scene_words;
        changed_occupancy_scene_words[457] = 2;
        let changed_occupancy_words =
            run_trace_schedule_shader(&device, &queue, &changed_occupancy_scene_words, &hzb_view);
        assert_ne!(
            changed_occupancy_words[70] & 1023,
            words[70] & 1023,
            "voxel occupancy changes must invalidate the local temporal support signature"
        );
    }

    #[test]
    fn trace_schedule_shader_consumes_hzb_tiles_and_inverse_view_projection() {
        let source = include_str!("../hybrid_gi/renderer/shaders/trace_schedule_handoff.wgsl");

        assert!(source.contains("SCENE_HZB_TILE_WORD_OFFSET"));
        assert!(source.contains("TRACE_HZB_TILE_WORD_OFFSET"));
        assert!(source.contains("reconstruct_world_position"));
        assert!(source.contains("let clip_depth = f32(depth_q24)"));
        assert!(source.contains("HYBRID_GI_HZB_TRACE_MAGIC"));
        assert!(source.contains("center_closest_depth_q24"));
        assert!(source.contains("center_furthest_depth_q24"));
        assert!(source.contains("trace_main_scene_hzb_ray"));
        assert!(source.contains("hzb_range_overlaps_ray_segment"));
        assert!(source.contains("surface_cache_radiance_for_world_position"));
        assert!(source.contains("voxel_radiance_for_world_position"));
        assert!(source.contains("TRACE_SURFACE_CACHE_HIT_FLAG"));
        assert!(source.contains("TRACE_VOXEL_FALLBACK_FLAG"));
        assert!(source.contains("SCENE_NORMAL_CODE_SHIFT"));
        assert!(source.contains("trace_tile_offset + 7u"));
    }

    fn quantize_depth_q24(depth: f32) -> u32 {
        (depth.clamp(0.0, 1.0) * DEPTH_Q24_MAX as f32 + 0.5) as u32
    }

    fn test_scene_words(start_depth_q24: u32, target_depth_q24: u32) -> Vec<u32> {
        let mut scene_words = vec![0_u32; SCENE_WORD_COUNT];
        scene_words[0..16].copy_from_slice(&[
            SCENE_DEPTH_HANDOFF_MAGIC,
            16,
            16,
            target_depth_q24,
            1,
            8,
            8,
            4,
            quantize_depth_q24(0.9),
            start_depth_q24,
            quantize_depth_q24(0.9),
            start_depth_q24,
            1 << 31,
            0,
            8,
            64,
        ]);
        for tile_index in 0..64 {
            let offset = 16 + tile_index * 4;
            scene_words[offset] = target_depth_q24;
            scene_words[offset + 1] = quantize_depth_q24(0.9);
            scene_words[offset + 2] = start_depth_q24;
            scene_words[offset + 3] = (1 << 31) | (DEFAULT_NORMAL_CODE << 8);
        }
        scene_words[16] = start_depth_q24;
        scene_words[272] = CAMERA_PACKET_MAGIC;
        for diagonal in [0_usize, 5, 10, 15] {
            scene_words[273 + diagonal] = 1.0_f32.to_bits();
        }
        scene_words[292..294].copy_from_slice(&[16, 16]);
        scene_words[294..302].copy_from_slice(&[
            SCENE_TRACE_INPUT_MAGIC,
            0,
            0,
            0,
            302,
            430,
            454,
            0x1234_5678,
        ]);
        scene_words
    }

    fn run_trace_schedule_shader(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_words: &[u32],
        hzb_view: &wgpu::TextureView,
    ) -> Vec<u32> {
        let scene = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("hybrid-gi-trace-schedule-test-scene"),
            contents: bytemuck::cast_slice(scene_words),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let trace = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hybrid-gi-trace-schedule-test-trace"),
            size: TRACE_WORD_COUNT as u64 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hybrid-gi-trace-schedule-test-readback"),
            size: TRACE_WORD_COUNT as u64 * std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("hybrid-gi-trace-schedule-test"),
        });
        let bind_group_layout = create_trace_schedule_handoff_bind_group_layout(device);
        let pipeline = create_trace_schedule_handoff_pipeline(device, &bind_group_layout);
        let bind_group = create_trace_schedule_handoff_bind_group(
            device,
            &bind_group_layout,
            &scene,
            &trace,
            hzb_view,
        );
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("hybrid-gi-trace-schedule-test-compute"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        encoder.copy_buffer_to_buffer(
            &trace,
            0,
            &readback,
            0,
            TRACE_WORD_COUNT as u64 * std::mem::size_of::<u32>() as u64,
        );
        queue.submit([encoder.finish()]);
        read_u32_words(device, &readback, TRACE_WORD_COUNT)
    }

    fn test_main_scene_hzb(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        hit_depth: f32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("hybrid-gi-main-scene-hzb-march-test-texture"),
            size: wgpu::Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 1,
            },
            mip_level_count: 4,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let mip_x_ranges = [
            vec![
                (0.25, 0.25),
                (0.9, 0.9),
                (0.9, 0.9),
                (0.9, 0.9),
                (hit_depth, hit_depth),
                (0.9, 0.9),
                (0.9, 0.9),
                (0.9, 0.9),
            ],
            vec![(0.9, 0.25), (0.9, 0.9), (0.9, hit_depth), (0.9, 0.9)],
            vec![(0.9, 0.25), (0.9, hit_depth)],
            vec![(0.9, 0.25)],
        ];
        for (mip_level, x_ranges) in mip_x_ranges.iter().enumerate() {
            let extent = 8_u32 >> mip_level;
            let mut pixels = Vec::with_capacity((extent * extent) as usize);
            for _ in 0..extent {
                pixels.extend(
                    x_ranges.iter().map(|&(furthest, closest)| {
                        [furthest, closest, furthest - closest, 1.0_f32]
                    }),
                );
            }
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: mip_level as u32,
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
            label: Some("hybrid-gi-main-scene-hzb-march-test-view"),
            base_mip_level: 0,
            mip_level_count: Some(4),
            ..Default::default()
        });
        (texture, view)
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
            label: Some("zircon-hybrid-gi-trace-schedule-test-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }))
        .ok()
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
