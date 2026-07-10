use super::*;
use std::fs;
use std::path::PathBuf;

use image::{ImageBuffer, ImageFormat, Rgba};

const MATRIX_SIDE: u32 = 128;
const MATRIX_CELL_SIDE: u32 = MATRIX_SIDE / 4;
const MATRIX_DISPLAY_EXPOSURE: u16 = 6;
const MULTI_DIRECTION_TRACE_QUALITY_WGPU_PNG: &str =
    "plan18_hybrid_gi_multi_direction_trace_quality_wgpu_20260711.png";
const MULTI_DIRECTION_TRACE_QUALITY_WGPU_REPORT: &str =
    "plan18_hybrid_gi_multi_direction_trace_quality_wgpu_20260711.txt";

#[test]
fn high_quality_trace_tiles_sample_directions_outside_the_low_quality_quartet() {
    let Some((low, high)) = trace_multi_ray_quality_samples() else {
        eprintln!("skipping multi-ray quality Wgpu test because no adapter is available");
        return;
    };

    let low_red = low & 0xff;
    let high_red = high & 0xff;
    assert!(
        high_red > low_red + 5,
        "High quality should sample the +X texel outside Low's four-direction set; low={low:#08x}, high={high:#08x}"
    );
}

fn trace_multi_ray_quality_samples() -> Option<(u32, u32)> {
    let (device, queue) = test_device()?;

    let mut atlas_pixels = vec![[0, 0, 0, 0]; 25];
    let mut depth_pixels = vec![[255, 255, 255, 0]; 25];
    atlas_pixels[12] = [20, 20, 20, 255];
    depth_pixels[12] = [128, 128, 128, 255];
    atlas_pixels[13] = [220, 20, 20, 255];
    depth_pixels[13] = [132, 132, 132, 255];

    let (_atlas_texture, atlas_view) = create_test_surface_cache_texture_with_pixels(
        &device,
        &queue,
        "zircon-hybrid-gi-multi-ray-quality-atlas",
        5,
        5,
        &atlas_pixels,
    );
    let (_depth_texture, depth_view) = create_test_surface_cache_texture_with_pixels(
        &device,
        &queue,
        "zircon-hybrid-gi-multi-ray-quality-depth",
        5,
        5,
        &depth_pixels,
    );

    let low = trace_surface_cache_tile(
        &device,
        &queue,
        &atlas_view,
        &depth_view,
        4,
        5,
        12,
        "zircon-hybrid-gi-multi-ray-quality-low",
    );
    let high = trace_surface_cache_tile(
        &device,
        &queue,
        &atlas_view,
        &depth_view,
        16,
        5,
        12,
        "zircon-hybrid-gi-multi-ray-quality-high",
    );

    Some((low, high))
}

fn trace_multi_ray_direction_matrix() -> Option<([u32; 16], [u32; 16])> {
    let (device, queue) = test_device()?;
    let directions = [
        [1_i32, 0_i32],
        [0, 1],
        [-1, 0],
        [0, -1],
        [1, 1],
        [-1, 1],
        [-1, -1],
        [1, -1],
        [2, 1],
        [1, 2],
        [-1, 2],
        [-2, 1],
        [-2, -1],
        [-1, -2],
        [1, -2],
        [2, -1],
    ];
    let mut low_samples = [0_u32; 16];
    let mut high_samples = [0_u32; 16];

    for (direction_index, [direction_x, direction_y]) in directions.into_iter().enumerate() {
        let mut atlas_pixels = vec![[0, 0, 0, 0]; 81];
        let mut depth_pixels = vec![[255, 255, 255, 0]; 81];
        atlas_pixels[40] = [20, 20, 20, 255];
        depth_pixels[40] = [128, 128, 128, 255];
        let hit_x = (4 + direction_x) as usize;
        let hit_y = (4 + direction_y) as usize;
        let hit_index = hit_y * 9 + hit_x;
        atlas_pixels[hit_index] = [220, 40, 20, 255];
        depth_pixels[hit_index] = [132, 132, 132, 255];

        let (_atlas_texture, atlas_view) = create_test_surface_cache_texture_with_pixels(
            &device,
            &queue,
            "zircon-hybrid-gi-multi-ray-matrix-atlas",
            9,
            9,
            &atlas_pixels,
        );
        let (_depth_texture, depth_view) = create_test_surface_cache_texture_with_pixels(
            &device,
            &queue,
            "zircon-hybrid-gi-multi-ray-matrix-depth",
            9,
            9,
            &depth_pixels,
        );
        low_samples[direction_index] = trace_surface_cache_tile(
            &device,
            &queue,
            &atlas_view,
            &depth_view,
            4,
            9,
            40,
            "zircon-hybrid-gi-multi-ray-matrix-low",
        );
        high_samples[direction_index] = trace_surface_cache_tile(
            &device,
            &queue,
            &atlas_view,
            &depth_view,
            16,
            9,
            40,
            "zircon-hybrid-gi-multi-ray-matrix-high",
        );
    }

    Some((low_samples, high_samples))
}

#[test]
#[ignore]
fn export_multi_direction_trace_quality_wgpu_png() {
    let Some((low_samples, high_samples)) = trace_multi_ray_direction_matrix() else {
        eprintln!("skipping multi-direction quality Wgpu product because no adapter is available");
        return;
    };

    let center_rgb = unpack_packed_rgb8(low_samples[0]);
    let low_hit_cells = low_samples
        .iter()
        .filter(|sample| unpack_packed_rgb8(**sample)[0] > center_rgb[0] + 5)
        .count();
    let high_hit_cells = high_samples
        .iter()
        .filter(|sample| unpack_packed_rgb8(**sample)[0] > center_rgb[0] + 5)
        .count();
    let different_cells = low_samples
        .iter()
        .zip(high_samples)
        .filter(|(low, high)| **low != *high)
        .count();

    assert_eq!(
        low_hit_cells, 4,
        "Low quality should trace its four deterministic directions"
    );
    assert_eq!(
        high_hit_cells, 16,
        "High quality should trace the complete sixteen-direction set"
    );
    assert!(
        different_cells >= 12,
        "High quality should add at least twelve direction hits beyond Low; different_cells={different_cells}"
    );

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_direction_matrix_png(
        output_dir.join(MULTI_DIRECTION_TRACE_QUALITY_WGPU_PNG),
        &low_samples,
        &high_samples,
    );
    fs::write(
        output_dir.join(MULTI_DIRECTION_TRACE_QUALITY_WGPU_REPORT),
        format!(
            "png={}\nleft=low_quality_4_direction_rays\nright=high_quality_16_direction_rays\nwidth={}\nheight={}\ngpu_output_grid=4x4_direction_target_matrix\ndisplay_exposure={}x_uniform_rgb\nlow_hit_cells={}\nhigh_hit_cells={}\ndifferent_cells={}\nlow_quality_tracing_budget=8\nhigh_quality_tracing_budget=32\nlow_surface_cache_rays_per_trace_tile=4\nhigh_surface_cache_rays_per_trace_tile=16\ndirection_set=16_deterministic_axis_diagonal_and_intermediate_screen_directions\ndirectional_trace=per_direction_hzb_march_then_equal_weight_radiance_aggregation\ngpu_pipeline=trace_probe_tiles_compute+surface_cache_atlas_depth_texture_load\nvalidated_quality_expansion=high_quality_trace_tiles_sample_directions_outside_the_low_quality_quartet\nvalidated_shader_regression=9_trace_probe_tiles_shader_tests\nlumen_reference=GenerateRays_EquiAreaSphericalMapping_screen_probe_tracing_octahedron_plus_TraceScreen_ray_texel_trace\nlow_packed_rgb={:?}\nhigh_packed_rgb={:?}\n",
            MULTI_DIRECTION_TRACE_QUALITY_WGPU_PNG,
            MATRIX_SIDE * 2 + 1,
            MATRIX_SIDE,
            MATRIX_DISPLAY_EXPOSURE,
            low_hit_cells,
            high_hit_cells,
            different_cells,
            low_samples,
            high_samples,
        ),
    )
    .unwrap();
}

fn write_direction_matrix_png(path: PathBuf, low_samples: &[u32; 16], high_samples: &[u32; 16]) {
    let output_width = MATRIX_SIDE * 2 + 1;
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(output_width, MATRIX_SIDE);
    for (panel_index, samples) in [low_samples, high_samples].into_iter().enumerate() {
        let panel_x = panel_index as u32 * (MATRIX_SIDE + 1);
        for (cell_index, sample) in samples.iter().enumerate() {
            let [red, green, blue] = unpack_packed_rgb8(*sample).map(apply_matrix_display_exposure);
            let cell_x = cell_index as u32 % 4;
            let cell_y = cell_index as u32 / 4;
            for y in 0..MATRIX_CELL_SIDE {
                for x in 0..MATRIX_CELL_SIDE {
                    let border = x == 0 || y == 0;
                    let pixel = if border {
                        Rgba([6, 8, 10, 255])
                    } else {
                        Rgba([red, green, blue, 255])
                    };
                    image.put_pixel(
                        panel_x + cell_x * MATRIX_CELL_SIDE + x,
                        cell_y * MATRIX_CELL_SIDE + y,
                        pixel,
                    );
                }
            }
        }
    }
    for y in 0..MATRIX_SIDE {
        image.put_pixel(MATRIX_SIDE, y, Rgba([255, 255, 255, 255]));
    }
    image.save_with_format(path, ImageFormat::Png).unwrap();
}

fn unpack_packed_rgb8(packed: u32) -> [u8; 3] {
    [
        (packed & 0xff) as u8,
        ((packed >> 8) & 0xff) as u8,
        ((packed >> 16) & 0xff) as u8,
    ]
}

fn apply_matrix_display_exposure(channel: u8) -> u8 {
    (u16::from(channel) * MATRIX_DISPLAY_EXPOSURE).min(255) as u8
}

fn render_test_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
}

fn trace_surface_cache_tile(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    atlas_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
    ray_count: u32,
    atlas_extent: u32,
    tile_sample_id: u32,
    label: &'static str,
) -> u32 {
    let params_buffer = create_probe_trace_tile_dispatch_params_buffer(
        device,
        1,
        0,
        1,
        ProbeTraceTileSurfaceCacheParams {
            texture_available: 1,
            atlas_width: atlas_extent,
            atlas_height: atlas_extent,
            atlas_columns: atlas_extent,
            tile_extent: 1,
        },
        0,
    );
    let resident_probe_buffer = create_storage_buffer(device, label, &[resident_probe_input(7)]);
    let pending_probe_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-multi-ray-quality-pending-probe",
        &[GpuPendingProbeInput::zeroed()],
    );
    let probe_trace_tile_buffer = create_storage_buffer(
        device,
        "zircon-hybrid-gi-multi-ray-quality-tile",
        &[0_u32, 7, tile_sample_id, ray_count],
    );
    let trace_lighting_buffer = create_trace_lighting_buffer(device, 3);
    let readback_buffer = create_readback_buffer(device, 3);
    let scene_prepare_descriptor_buffer = create_zeroed_scene_prepare_descriptor_buffer(device);
    let bind_group_layout = create_probe_trace_tile_dispatch_bind_group_layout(device);
    let pipeline = create_probe_trace_tile_dispatch_pipeline(device, &bind_group_layout);
    let bind_group = create_probe_trace_tile_dispatch_bind_group(
        device,
        &bind_group_layout,
        &params_buffer,
        &resident_probe_buffer,
        &pending_probe_buffer,
        &probe_trace_tile_buffer,
        &trace_lighting_buffer,
        atlas_view,
        depth_view,
        &scene_prepare_descriptor_buffer,
    );

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
    encode_probe_trace_tile_dispatch(&mut encoder, &pipeline, &bind_group, 1);
    encoder.copy_buffer_to_buffer(
        &trace_lighting_buffer,
        0,
        &readback_buffer,
        0,
        (3 * std::mem::size_of::<u32>()) as u64,
    );
    queue.submit(std::iter::once(encoder.finish()));

    let words = readback_u32s(device, &readback_buffer, 3);
    assert_eq!(words[0], 1);
    assert_eq!(words[1], 7);
    words[2]
}
