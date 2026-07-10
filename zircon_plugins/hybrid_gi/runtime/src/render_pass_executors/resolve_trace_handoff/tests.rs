use std::sync::mpsc;

use wgpu::util::DeviceExt;

use super::*;

const TRACE_WORD_COUNT: usize = 512;
const TRACE_TILE_WORD_OFFSET: usize = 64;
const TRACE_TILE_WORD_COUNT: usize = 7;
const TRACE_TILE_COUNT: usize = 64;
const TRACE_SCHEDULE_MAGIC: u32 = 0x4847_4954;
const HZB_TRACE_MAGIC: u32 = 0x4847_5a42;
const SURFACE_CACHE_FLAG: u32 = 1 << 10;
const RADIANCE_VALID_FLAG: u32 = 1 << 12;
const CURRENT_RADIANCE: [u8; 4] = [64, 128, 192, 255];
const TEST_SIZE: u32 = 4;
const READBACK_BYTES_PER_ROW: u32 = 256;

#[test]
fn resolve_shader_consumes_trace_depth_source_packet() {
    let source = include_str!("../../hybrid_gi/renderer/shaders/resolve_trace_depth_source.wgsl");

    assert!(source.contains("HYBRID_GI_TRACE_SCHEDULE_MAGIC"));
    assert!(source.contains("hybrid_gi_trace_words[6]"));
    assert!(source.contains("@fragment"));
    assert!(source.contains("unpack_rgba8"));
    assert!(source.contains("HYBRID_GI_HZB_TRACE_MAGIC"));
    assert!(source.contains("TRACE_HZB_TILE_WORD_OFFSET"));
    assert!(source.contains("trace_tile_coord"));
    assert!(source.contains("scene_velocity_tex"));
    assert!(source.contains("previous_temporal_metadata_tex"));
    assert!(source.contains("temporal_history_weight"));
    assert!(source.contains("normalized_support_signature"));
    assert!(source.contains("HybridGiTemporalResolveOutput"));
}

#[test]
fn resolve_temporal_history_reuses_static_scene_and_accumulates_confidence() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let baseline = run_temporal_resolve(
        &device,
        &queue,
        TemporalCase::new(false, [0.0, 0.0], 0.0, 1.0),
    );
    let reused = run_temporal_resolve(
        &device,
        &queue,
        TemporalCase::new(true, [0.0, 0.0], 0.0, 1.0),
    );

    assert_color_matches_current(baseline.lighting);
    assert!(
        reused.lighting[0] > baseline.lighting[0] + 0.05,
        "static history should contribute to resolved GI: baseline={:?}, reused={:?}",
        baseline.lighting,
        reused.lighting
    );
    assert!(
        reused.metadata[3] > baseline.metadata[3] + 0.5,
        "accepted history should accumulate confidence: baseline={:?}, reused={:?}",
        baseline.metadata,
        reused.metadata
    );
}

#[test]
fn resolve_temporal_history_rejects_reprojected_motion() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let baseline = run_temporal_resolve(
        &device,
        &queue,
        TemporalCase::new(false, [0.0, 0.0], 0.0, 1.0),
    );
    let moving = run_temporal_resolve(
        &device,
        &queue,
        TemporalCase::new(true, [2.0, 0.0], 0.0, 1.0),
    );

    assert_vec4_near(moving.lighting, baseline.lighting, 0.01);
    assert!(
        moving.metadata[3] <= 0.3,
        "motion rejection should reset confidence"
    );
}

#[test]
fn resolve_temporal_history_rejects_scene_signature_or_trace_source_change() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let baseline = run_temporal_resolve(
        &device,
        &queue,
        TemporalCase::new(false, [0.0, 0.0], 0.0, 1.0),
    );
    let changed_scene = run_temporal_resolve(
        &device,
        &queue,
        TemporalCase::new(true, [0.0, 0.0], 1.0, 1.0),
    );
    let changed_source = run_temporal_resolve(
        &device,
        &queue,
        TemporalCase::new(true, [0.0, 0.0], 0.0, 2.0),
    );

    assert_vec4_near(changed_scene.lighting, baseline.lighting, 0.01);
    assert_vec4_near(changed_source.lighting, baseline.lighting, 0.01);
    assert!(changed_scene.metadata[3] <= 0.3);
    assert!(changed_source.metadata[3] <= 0.3);
}

#[test]
fn resolve_temporal_history_reuses_unchanged_support_and_rejects_changed_neighbor() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let baseline = run_temporal_resolve_pixels(
        &device,
        &queue,
        TemporalCase::new(false, [0.0, 0.0], 0.0, 1.0),
    );
    let unchanged_signature = 128_u32;
    let changed_signature = 768_u32;
    let previous_signature = normalized_signature(unchanged_signature);
    let mut current_signatures = [unchanged_signature; (TEST_SIZE * TEST_SIZE) as usize];
    let changed_pixel_index = 6;
    current_signatures[changed_pixel_index] = changed_signature;
    let localized = run_temporal_resolve_pixels(
        &device,
        &queue,
        TemporalCase::new(true, [0.0, 0.0], previous_signature, 1.0)
            .with_current_support_signatures(current_signatures),
    );

    for pixel_index in [5, 9, 10] {
        assert!(
            localized[pixel_index].lighting[0] > baseline[pixel_index].lighting[0] + 0.05,
            "unchanged support should retain local history at pixel {pixel_index}: baseline={:?}, localized={:?}",
            baseline[pixel_index].lighting,
            localized[pixel_index].lighting,
        );
        assert!(localized[pixel_index].metadata[3] > 0.75);
    }
    assert_vec4_near(
        localized[changed_pixel_index].lighting,
        baseline[changed_pixel_index].lighting,
        0.01,
    );
    assert!(
        localized[changed_pixel_index].metadata[3] <= 0.3,
        "changed support should reset only its local confidence"
    );
}

#[derive(Clone, Copy)]
struct TemporalCase {
    history_available: bool,
    velocity: [f32; 2],
    previous_signature: f32,
    previous_source: f32,
    current_support_signatures: [u32; (TEST_SIZE * TEST_SIZE) as usize],
}

impl TemporalCase {
    const fn new(
        history_available: bool,
        velocity: [f32; 2],
        previous_signature: f32,
        previous_source: f32,
    ) -> Self {
        Self {
            history_available,
            velocity,
            previous_signature,
            previous_source,
            current_support_signatures: [0; (TEST_SIZE * TEST_SIZE) as usize],
        }
    }

    const fn with_current_support_signatures(
        mut self,
        signatures: [u32; (TEST_SIZE * TEST_SIZE) as usize],
    ) -> Self {
        self.current_support_signatures = signatures;
        self
    }
}

#[derive(Clone, Copy)]
struct TemporalResult {
    lighting: [f32; 4],
    metadata: [f32; 4],
}

fn run_temporal_resolve(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    case: TemporalCase,
) -> TemporalResult {
    run_temporal_resolve_pixels(device, queue, case)[0]
}

fn run_temporal_resolve_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    case: TemporalCase,
) -> Vec<TemporalResult> {
    let trace_words = test_trace_words(case.current_support_signatures);
    let trace = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("hybrid-gi-temporal-test-trace"),
        contents: bytemuck::cast_slice(&trace_words),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let (_velocity, velocity_view) = sampled_rg32_texture(
        device,
        queue,
        "hybrid-gi-temporal-test-velocity",
        [case.velocity[0], case.velocity[1]],
    );
    let (_history, history_view) = sampled_rgba32_texture(
        device,
        queue,
        "hybrid-gi-temporal-test-history",
        [0.375, 0.5, 0.75, 1.0],
    );
    let (_metadata_history, metadata_history_view) = sampled_rgba32_texture(
        device,
        queue,
        "hybrid-gi-temporal-test-metadata-history",
        [0.5, case.previous_source, case.previous_signature, 1.0],
    );
    let (lighting, lighting_view) = render_target(device, "hybrid-gi-temporal-test-lighting");
    let (metadata, metadata_view) = render_target(device, "hybrid-gi-temporal-test-metadata");
    let params = HybridGiTemporalResolveParams::new([TEST_SIZE, TEST_SIZE], case.history_available);
    let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("hybrid-gi-temporal-test-params"),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let layout = create_resolve_trace_bind_group_layout(device);
    let pipeline = create_resolve_trace_pipeline(
        device,
        &layout,
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureFormat::Rgba16Float,
        1,
    );
    let bind_group = create_resolve_trace_bind_group(
        device,
        &layout,
        &trace,
        &velocity_view,
        &history_view,
        &metadata_history_view,
        &params_buffer,
    );
    let lighting_readback = readback_buffer(device, "hybrid-gi-temporal-test-lighting-readback");
    let metadata_readback = readback_buffer(device, "hybrid-gi-temporal-test-metadata-readback");
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("hybrid-gi-temporal-test-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("HybridGiTemporalTestPass"),
            color_attachments: &[
                Some(color_attachment(&lighting_view)),
                Some(color_attachment(&metadata_view)),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    copy_target_to_buffer(&mut encoder, &lighting, &lighting_readback);
    copy_target_to_buffer(&mut encoder, &metadata, &metadata_readback);
    queue.submit([encoder.finish()]);

    let lighting_pixels = read_rgba16_pixels(device, &lighting_readback);
    let metadata_pixels = read_rgba16_pixels(device, &metadata_readback);
    lighting_pixels
        .into_iter()
        .zip(metadata_pixels)
        .map(|(lighting, metadata)| TemporalResult { lighting, metadata })
        .collect()
}

fn test_trace_words(
    current_support_signatures: [u32; (TEST_SIZE * TEST_SIZE) as usize],
) -> [u32; TRACE_WORD_COUNT] {
    let mut words = [0_u32; TRACE_WORD_COUNT];
    words[0] = TRACE_SCHEDULE_MAGIC;
    words[1] = TRACE_TILE_COUNT as u32;
    words[10] = HZB_TRACE_MAGIC;
    words[19] = 1;
    words[20] = 8;
    words[21] = TRACE_TILE_COUNT as u32;
    words[44] = TRACE_TILE_WORD_OFFSET as u32;
    words[45] = TRACE_TILE_WORD_COUNT as u32;
    words[49] = 0;
    for tile_index in 0..TRACE_TILE_COUNT {
        let offset = TRACE_TILE_WORD_OFFSET + tile_index * TRACE_TILE_WORD_COUNT;
        words[offset] = pack_rgba8(CURRENT_RADIANCE);
        words[offset + 1] = quantize_depth_q24(0.5);
        words[offset + 3] = SURFACE_CACHE_FLAG | RADIANCE_VALID_FLAG;
    }
    for (pixel_index, signature) in current_support_signatures.into_iter().enumerate() {
        let pixel_x = pixel_index as u32 % TEST_SIZE;
        let pixel_y = pixel_index as u32 / TEST_SIZE;
        let tile_x = ((pixel_x * 2 + 1) * 8) / (TEST_SIZE * 2);
        let tile_y = ((pixel_y * 2 + 1) * 8) / (TEST_SIZE * 2);
        let tile_index = (tile_y * 8 + tile_x) as usize;
        let offset = TRACE_TILE_WORD_OFFSET + tile_index * TRACE_TILE_WORD_COUNT;
        words[offset + 6] = signature;
    }
    words
}

fn sampled_rg32_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &'static str,
    texel: [f32; 2],
) -> (wgpu::Texture, wgpu::TextureView) {
    let texels = vec![texel; (TEST_SIZE * TEST_SIZE) as usize];
    sampled_texture(
        device,
        queue,
        label,
        wgpu::TextureFormat::Rg32Float,
        bytemuck::cast_slice(&texels),
        8,
    )
}

fn sampled_rgba32_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &'static str,
    texel: [f32; 4],
) -> (wgpu::Texture, wgpu::TextureView) {
    let texels = vec![texel; (TEST_SIZE * TEST_SIZE) as usize];
    sampled_texture(
        device,
        queue,
        label,
        wgpu::TextureFormat::Rgba32Float,
        bytemuck::cast_slice(&texels),
        16,
    )
}

fn sampled_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &'static str,
    format: wgpu::TextureFormat,
    bytes: &[u8],
    bytes_per_texel: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: test_extent(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    queue.write_texture(
        texture.as_image_copy(),
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(TEST_SIZE * bytes_per_texel),
            rows_per_image: Some(TEST_SIZE),
        },
        test_extent(),
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn render_target(device: &wgpu::Device, label: &'static str) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: test_extent(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn color_attachment(view: &wgpu::TextureView) -> wgpu::RenderPassColorAttachment<'_> {
    wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        depth_slice: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
        },
    }
}

fn readback_buffer(device: &wgpu::Device, label: &'static str) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: u64::from(READBACK_BYTES_PER_ROW * TEST_SIZE),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn copy_target_to_buffer(
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    buffer: &wgpu::Buffer,
) {
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(READBACK_BYTES_PER_ROW),
                rows_per_image: Some(TEST_SIZE),
            },
        },
        test_extent(),
    );
}

fn read_rgba16_pixels(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Vec<[f32; 4]> {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).ok();
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("device poll should complete temporal readback");
    receiver
        .recv()
        .expect("temporal readback callback should run")
        .expect("temporal readback mapping should succeed");
    let mapped = slice.get_mapped_range();
    let mut pixels = Vec::with_capacity((TEST_SIZE * TEST_SIZE) as usize);
    for y in 0..TEST_SIZE as usize {
        let row_offset = y * READBACK_BYTES_PER_ROW as usize;
        for x in 0..TEST_SIZE as usize {
            let pixel_offset = row_offset + x * 8;
            let words = bytemuck::cast_slice::<u8, u16>(&mapped[pixel_offset..pixel_offset + 8]);
            pixels.push([
                f16_bits_to_f32(words[0]),
                f16_bits_to_f32(words[1]),
                f16_bits_to_f32(words[2]),
                f16_bits_to_f32(words[3]),
            ]);
        }
    }
    drop(mapped);
    buffer.unmap();
    pixels
}

fn test_extent() -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: TEST_SIZE,
        height: TEST_SIZE,
        depth_or_array_layers: 1,
    }
}

fn quantize_depth_q24(depth: f32) -> u32 {
    (depth.clamp(0.0, 1.0) * 16_777_215.0 + 0.5) as u32
}

fn normalized_signature(signature: u32) -> f32 {
    (signature & 1023) as f32 / 1023.0
}

const fn pack_rgba8(rgba8: [u8; 4]) -> u32 {
    rgba8[0] as u32
        | ((rgba8[1] as u32) << 8)
        | ((rgba8[2] as u32) << 16)
        | ((rgba8[3] as u32) << 24)
}

fn assert_color_matches_current(actual: [f32; 4]) {
    let expected = [
        f32::from(CURRENT_RADIANCE[0]) / 255.0,
        f32::from(CURRENT_RADIANCE[1]) / 255.0,
        f32::from(CURRENT_RADIANCE[2]) / 255.0,
        1.0,
    ];
    assert_vec4_near(actual, expected, 0.01);
}

fn assert_vec4_near(actual: [f32; 4], expected: [f32; 4], tolerance: f32) {
    for channel in 0..4 {
        assert!(
            (actual[channel] - expected[channel]).abs() <= tolerance,
            "channel {channel} mismatch: actual={actual:?}, expected={expected:?}, tolerance={tolerance}"
        );
    }
}

fn f16_bits_to_f32(bits: u16) -> f32 {
    let sign = u32::from(bits >> 15) << 31;
    let exponent = u32::from((bits >> 10) & 0x1f);
    let mantissa = u32::from(bits & 0x03ff);
    let f32_bits = match exponent {
        0 if mantissa == 0 => sign,
        0 => {
            let mut normalized = mantissa;
            let mut shift = 0_u32;
            while normalized & 0x0400 == 0 {
                normalized <<= 1;
                shift += 1;
            }
            sign | ((113_u32.saturating_sub(shift)) << 23) | ((normalized & 0x03ff) << 13)
        }
        0x1f => sign | 0x7f80_0000 | (mantissa << 13),
        _ => sign | ((exponent + 112) << 23) | (mantissa << 13),
    };
    f32::from_bits(f32_bits)
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
        label: Some("zircon-hybrid-gi-temporal-resolve-test-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
}
