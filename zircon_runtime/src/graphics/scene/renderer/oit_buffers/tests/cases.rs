use super::*;

#[test]
fn oit_draw_shader_uses_per_pixel_atomic_count_and_fixed_layer_stride() {
    assert!(OIT_DRAW_SHADER_SOURCE.contains("atomicAdd(&oit_counts[pixel_index], 1u)"));
    assert!(OIT_DRAW_SHADER_SOURCE.contains("pixel_index * oit_settings.fragments_per_pixel"));
    assert!(OIT_DRAW_SHADER_SOURCE.contains("pack4x8unorm"));
    assert!(OIT_DRAW_SHADER_SOURCE.contains("bitcast<u32>(position.z)"));
    assert!(OIT_DRAW_SHADER_SOURCE.contains("physical_pixel - origin"));
    assert!(OIT_DRAW_SHADER_SOURCE
        .contains("@group(4) @binding(0) var<storage, read_write> oit_layers"));
    assert!(!OIT_DRAW_SHADER_SOURCE.contains("@group(3)"));
}

#[test]
fn oit_resolve_shader_sorts_near_to_far_depth_and_bounds_exact_layers() {
    assert!(OIT_RESOLVE_SHADER_SOURCE.contains("const OIT_MAX_SORTED_FRAGMENTS: u32 = 32u"));
    assert!(OIT_RESOLVE_SHADER_SOURCE.contains("candidate.y < sorted_layers[j - 1u].y"));
    assert!(OIT_RESOLVE_SHADER_SOURCE.contains("min(oit_settings.sorted_fragment_max_count"));
    assert!(OIT_RESOLVE_SHADER_SOURCE.contains("physical_pixel - origin"));
}

#[test]
fn oit_fragment_store_clears_only_the_atomic_count_buffer() {
    let source = include_str!("../executors.rs");

    assert!(source.contains("clear_buffer(\n            counts.buffer,"));
    assert!(source.contains("counts.size.map(std::num::NonZeroU64::get)"));
    assert!(!source.contains("clear_buffer(&layers, 0, None)"));
}

fn regression_device() -> (wgpu::Device, wgpu::Queue) {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::PRIMARY;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .expect("this regression requires a native GPU adapter");
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("render-contract-regression"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .expect("this regression requires a native GPU device")
}

fn regression_texture(
    device: &wgpu::Device,
    width: u32,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("render-contract-regression"),
        size: wgpu::Extent3d {
            width,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn read_regression_row(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut encoder: wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    width: u32,
    byte_count: usize,
) -> Vec<u8> {
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("render-contract-readback"),
        size: 256,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: Some(1),
            },
        },
        wgpu::Extent3d {
            width,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    queue.submit([encoder.finish()]);
    let slice = readback.slice(..);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(deadline.saturating_duration_since(std::time::Instant::now())),
        })
        .expect("GPU work must complete before the aggregate deadline");
    receiver
        .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
        .expect("readback callback must complete before the aggregate deadline")
        .expect("readback mapping must succeed");
    let mapped = slice.get_mapped_range();
    let bytes = mapped[..byte_count].to_vec();
    drop(mapped);
    readback.unmap();
    bytes
}

#[test]
fn oit_overflow_premultiplied_color_matches_the_exact_sorted_composite_on_gpu() {
    let (device, queue) = regression_device();
    let pipeline =
        super::resolve_pipeline::OitResolvePipeline::new(&device, wgpu::TextureFormat::Rgba8Unorm);
    let layers = wgpu::util::DeviceExt::create_buffer_init(
        &device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("oit-two-half-alpha-red-fragments"),
            contents: bytemuck::cast_slice(&[
                [0x800000ff_u32, 0.2_f32.to_bits()],
                [0x800000ff, 0.8_f32.to_bits()],
                [0, 0],
                [0, 0],
            ]),
            usage: wgpu::BufferUsages::STORAGE,
        },
    );
    let counts = wgpu::util::DeviceExt::create_buffer_init(
        &device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("oit-two-fragment-count"),
            contents: bytemuck::cast_slice(&[2_u32]),
            usage: wgpu::BufferUsages::STORAGE,
        },
    );
    let mut results = Vec::new();
    for sorted_fragment_max_count in [1, 2] {
        let output = regression_texture(&device, 1, wgpu::TextureFormat::Rgba8Unorm);
        let view = output.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear-oit-regression-target"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        pipeline.encode(
            &device,
            &mut encoder,
            &view,
            wgpu::BufferBinding {
                buffer: &layers,
                offset: 0,
                size: None,
            },
            wgpu::BufferBinding {
                buffer: &counts,
                offset: 0,
                size: None,
            },
            crate::graphics::types::ViewportRenderRegion::full_target(glam::UVec2::new(1, 1)),
            crate::core::framework::render::OitSettings {
                fragments_per_pixel_average: 4.0,
                sorted_fragment_max_count,
                alpha_threshold: 0.0,
            },
        );
        let pixel = read_regression_row(&device, &queue, encoder, &output, 1, 4);
        let alpha = 128.0_f64 / 255.0;
        let expected = ((alpha + (1.0 - alpha) * alpha) * 255.0).round() as i16;
        assert!((i16::from(pixel[0]) - expected).abs() <= 1, "{pixel:?}");
        assert!((i16::from(pixel[3]) - expected).abs() <= 1, "{pixel:?}");
        assert_eq!(&pixel[1..3], &[0, 0]);
        results.push(pixel);
    }
    assert_eq!(
        results[0], results[1],
        "moving the second fragment into overflow must preserve its premultiplied contribution"
    );
}
