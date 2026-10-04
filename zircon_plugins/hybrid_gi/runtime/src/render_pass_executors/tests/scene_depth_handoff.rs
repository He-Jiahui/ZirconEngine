use std::sync::mpsc;

use super::*;

const HANDOFF_MAGIC: u32 = 0x48474944;
const DEPTH_Q24_SCALE: f32 = 16777215.0;

#[test]
fn scene_depth_handoff_records_buffer_writes_without_native_queue_authority() {
    let source = include_str!("../scene_depth_handoff.rs");
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
        include_str!("../../hybrid_gi/renderer/shaders/scene_depth_handoff.wgsl"),
        include_str!("../../hybrid_gi/renderer/shaders/scene_depth_handoff_msaa.wgsl"),
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
