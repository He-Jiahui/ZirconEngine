use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::core::framework::render::{FogVolumeData, RenderLayerSet};
use crate::core::math::Mat4;

use super::*;

const TEST_DIMENSIONS: [u32; 3] = [16, 8, 8];
const BYTES_PER_TEXEL: u32 = 8;
const READBACK_BYTES_PER_ROW: u32 = 256;
const PRODUCT_PNG: &str = "plan18_volumetric_media_inject_wgpu_20260711.png";
const PRODUCT_REPORT: &str = "plan18_volumetric_media_inject_wgpu_20260711.txt";

#[test]
fn render_volumetric_media_inject_upload_bytes_follow_local_volume_payload() {
    assert_eq!(FroxelMediaInjectPipeline::uploaded_bytes(0, true), 160);
    assert_eq!(FroxelMediaInjectPipeline::uploaded_bytes(8, false), 160);
    assert_eq!(FroxelMediaInjectPipeline::uploaded_bytes(2, true), 256);
}

#[test]
fn render_volumetric_media_inject_writes_global_and_local_froxel_media() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let result = run_media_inject(&device, &queue, true);
    let metrics = media_metrics(&result.texels);

    assert_eq!(result.dispatch, [4, 2, 2]);
    assert!(
        metrics.right_extinction > metrics.left_extinction + 0.08,
        "local volume should increase right-half extinction: {metrics:?}"
    );
    assert!(
        metrics.right_scattering[0] > metrics.left_scattering[0] + 0.15,
        "local volume should add red scattering: {metrics:?}"
    );
    assert!(
        metrics.left_scattering[2] > metrics.left_scattering[0] * 3.0,
        "global medium should preserve blue-dominant albedo: {metrics:?}"
    );

    let low_quality = run_media_inject(&device, &queue, false);
    let low_metrics = media_metrics(&low_quality.texels);
    assert!(
        (low_metrics.right_extinction - low_metrics.left_extinction).abs() < 0.002,
        "disabling local volumes should leave only global height fog: {low_metrics:?}"
    );
}

#[test]
fn render_volumetric_media_inject_shader_owns_rgba16f_3d_storage_contract() {
    let source = include_str!("shaders/media_inject.wgsl");

    assert!(source.contains("texture_storage_3d<rgba16float, write>"));
    assert!(source.contains("global_density"));
    assert!(MEDIA_INJECT_SHADER.contains("zr_froxel_world_position"));
    assert!(MEDIA_INJECT_SHADER.contains("world_from_clip"));
    assert!(source.contains("fog_volumes[volume_index]"));
    assert!(source.contains("textureStore(media_texture"));
    assert!(source.contains("@workgroup_size(4, 4, 4)"));
}

#[test]
#[ignore]
fn export_volumetric_media_inject_wgpu_png() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping volumetric media inject Wgpu product because no adapter is available");
        return;
    };
    let result = run_media_inject(&device, &queue, true);
    let metrics = media_metrics(&result.texels);
    assert!(metrics.right_extinction > metrics.left_extinction + 0.08);
    assert!(metrics.right_scattering[0] > metrics.left_scattering[0] + 0.15);

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_media_projection_png(output_dir.join(PRODUCT_PNG), &result.texels);
    fs::write(
        output_dir.join(PRODUCT_REPORT),
        format!(
            "png={PRODUCT_PNG}\nwidth=256\nheight=128\ngpu_froxel_dimensions=16x8x8\nprojection=xz_average_over_y\nformat=rgba16float\nmedia_rgb=scattering\nmedia_alpha=extinction\nworkgroup_size=4x4x4\ndispatch={},{},{}\nleft_global_only_average_extinction={:.6}\nright_global_plus_local_average_extinction={:.6}\nleft_average_scattering_rgb={:.6},{:.6},{:.6}\nright_average_scattering_rgb={:.6},{:.6},{:.6}\nlocal_volume_bounds=0,-100,-100_to_100,100,0\nfroxel_reconstruction=perspective_world_from_clip_exponential_view_depth\nquality_contract=low_disables_local_medium_high_enable_local_high_enables_temporal\nreference=UE_VolumetricFog_RWVBufferA_global_plus_local_media_injection\n",
            result.dispatch[0],
            result.dispatch[1],
            result.dispatch[2],
            metrics.left_extinction,
            metrics.right_extinction,
            metrics.left_scattering[0],
            metrics.left_scattering[1],
            metrics.left_scattering[2],
            metrics.right_scattering[0],
            metrics.right_scattering[1],
            metrics.right_scattering[2],
        ),
    )
    .unwrap();
}

struct MediaInjectResult {
    texels: Vec<[f32; 4]>,
    dispatch: [u32; 3],
}

fn run_media_inject(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    include_local_volumes: bool,
) -> MediaInjectResult {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-volumetric-media-inject-test-texture"),
        size: test_extent(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("zircon-volumetric-media-inject-test-view"),
        dimension: Some(wgpu::TextureViewDimension::D3),
        ..Default::default()
    });
    let local_volumes = [FogVolumeData {
        volume_id: 1,
        bounds_min: Vec3::new(0.0, -100.0, -100.0),
        bounds_max: Vec3::new(100.0, 100.0, 0.0),
        density: 0.12,
        albedo: Vec3::new(1.0, 0.2, 0.1),
        layer_mask: RenderLayerSet::default(),
    }];
    let request = FroxelMediaInjectRequest {
        settings: VolumetricFogSettings {
            density: 0.04,
            albedo: Vec3::new(0.2, 0.4, 1.0),
            phase_g: 0.3,
            height_falloff: 0.4,
            scattering_intensity: 2.0,
            depth_distribution_exp: 2.0,
            temporal: true,
        },
        grid: FroxelGridParams {
            dimensions: TEST_DIMENSIONS,
            near_depth: 0.1,
            far_depth: 100.0,
            depth_distribution_exp: 2.0,
        },
        view: test_froxel_view(),
        local_volumes: &local_volumes,
        include_local_volumes,
    };
    let pipeline = FroxelMediaInjectPipeline::new(device);
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-volumetric-media-inject-test-readback"),
        size: readback_size(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-volumetric-media-inject-test-encoder"),
    });
    let dispatch = pipeline
        .encode(device, &mut encoder, &view, request)
        .unwrap();
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(READBACK_BYTES_PER_ROW),
                rows_per_image: Some(TEST_DIMENSIONS[1]),
            },
        },
        test_extent(),
    );
    queue.submit([encoder.finish()]);
    MediaInjectResult {
        texels: read_rgba16f_3d(device, &readback),
        dispatch,
    }
}

fn test_froxel_view() -> FroxelViewReconstruction {
    FroxelViewReconstruction::perspective(
        Mat4::perspective_rh(90.0_f32.to_radians(), 2.0, 0.1, 100.0).inverse(),
        Vec3::ZERO,
        Vec3::NEG_Z,
    )
}

#[derive(Clone, Copy, Debug)]
struct MediaMetrics {
    left_extinction: f32,
    right_extinction: f32,
    left_scattering: [f32; 3],
    right_scattering: [f32; 3],
}

fn media_metrics(texels: &[[f32; 4]]) -> MediaMetrics {
    let mut left = [0.0_f32; 4];
    let mut right = [0.0_f32; 4];
    let mut left_count = 0_u32;
    let mut right_count = 0_u32;
    for z in 0..TEST_DIMENSIONS[2] {
        for y in 0..TEST_DIMENSIONS[1] {
            for x in 0..TEST_DIMENSIONS[0] {
                let sample = texels[texel_index(x, y, z)];
                let (sum, count) = if x < TEST_DIMENSIONS[0] / 2 {
                    (&mut left, &mut left_count)
                } else {
                    (&mut right, &mut right_count)
                };
                for channel in 0..4 {
                    sum[channel] += sample[channel];
                }
                *count += 1;
            }
        }
    }
    let left = left.map(|value| value / left_count as f32);
    let right = right.map(|value| value / right_count as f32);
    MediaMetrics {
        left_extinction: left[3],
        right_extinction: right[3],
        left_scattering: [left[0], left[1], left[2]],
        right_scattering: [right[0], right[1], right[2]],
    }
}

fn write_media_projection_png(path: PathBuf, texels: &[[f32; 4]]) {
    const CELL_WIDTH: u32 = 16;
    const CELL_HEIGHT: u32 = 16;
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(
        TEST_DIMENSIONS[0] * CELL_WIDTH,
        TEST_DIMENSIONS[2] * CELL_HEIGHT,
    );
    for z in 0..TEST_DIMENSIONS[2] {
        for x in 0..TEST_DIMENSIONS[0] {
            let mut scattering = [0.0_f32; 3];
            for y in 0..TEST_DIMENSIONS[1] {
                let sample = texels[texel_index(x, y, z)];
                for channel in 0..3 {
                    scattering[channel] += sample[channel];
                }
            }
            let color = scattering.map(|value| {
                ((value / TEST_DIMENSIONS[1] as f32 * 6.0).clamp(0.0, 1.0) * 255.0 + 0.5) as u8
            });
            for pixel_y in 0..CELL_HEIGHT {
                for pixel_x in 0..CELL_WIDTH {
                    image.put_pixel(
                        x * CELL_WIDTH + pixel_x,
                        z * CELL_HEIGHT + pixel_y,
                        Rgba(if pixel_x == 0 || pixel_y == 0 {
                            [5, 7, 9, 255]
                        } else {
                            [color[0], color[1], color[2], 255]
                        }),
                    );
                }
            }
        }
    }
    image.save_with_format(path, ImageFormat::Png).unwrap();
}

fn read_rgba16f_3d(device: &wgpu::Device, buffer: &wgpu::Buffer) -> Vec<[f32; 4]> {
    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).ok();
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("device poll should complete volumetric media readback");
    receiver
        .recv()
        .expect("volumetric media readback callback should run")
        .expect("volumetric media readback mapping should succeed");
    let mapped = slice.get_mapped_range();
    let mut texels =
        Vec::with_capacity((TEST_DIMENSIONS[0] * TEST_DIMENSIONS[1] * TEST_DIMENSIONS[2]) as usize);
    let image_stride = READBACK_BYTES_PER_ROW as usize * TEST_DIMENSIONS[1] as usize;
    for z in 0..TEST_DIMENSIONS[2] as usize {
        for y in 0..TEST_DIMENSIONS[1] as usize {
            let row_offset = z * image_stride + y * READBACK_BYTES_PER_ROW as usize;
            for x in 0..TEST_DIMENSIONS[0] as usize {
                let offset = row_offset + x * BYTES_PER_TEXEL as usize;
                let words = bytemuck::cast_slice::<u8, u16>(&mapped[offset..offset + 8]);
                texels.push([
                    f16_bits_to_f32(words[0]),
                    f16_bits_to_f32(words[1]),
                    f16_bits_to_f32(words[2]),
                    f16_bits_to_f32(words[3]),
                ]);
            }
        }
    }
    drop(mapped);
    buffer.unmap();
    texels
}

fn test_extent() -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: TEST_DIMENSIONS[0],
        height: TEST_DIMENSIONS[1],
        depth_or_array_layers: TEST_DIMENSIONS[2],
    }
}

fn readback_size() -> u64 {
    u64::from(READBACK_BYTES_PER_ROW * TEST_DIMENSIONS[1] * TEST_DIMENSIONS[2])
}

fn texel_index(x: u32, y: u32, z: u32) -> usize {
    (z * TEST_DIMENSIONS[0] * TEST_DIMENSIONS[1] + y * TEST_DIMENSIONS[0] + x) as usize
}

fn render_test_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
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
        label: Some("zircon-volumetric-media-inject-test-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
}
