use super::*;

const SPATIAL_FILTER_PRODUCT_PNG: &str =
    "plan18_hybrid_gi_depth_normal_support_spatial_denoise_wgpu_20260711.png";
const SPATIAL_FILTER_PRODUCT_REPORT: &str =
    "plan18_hybrid_gi_depth_normal_support_spatial_denoise_wgpu_20260711.txt";
const LEFT_SUPPORT_SIGNATURE: u32 = 128;
const RIGHT_SUPPORT_SIGNATURE: u32 = 768;

#[test]
fn resolve_spatial_filter_reduces_same_surface_noise_without_crossing_support_edge() {
    let Some((device, queue)) = test_device() else {
        return;
    };
    let (trace_words, raw_pixels) = spatial_filter_trace_words();
    let filtered = run_temporal_resolve_pixels_with_trace_words(
        &device,
        &queue,
        TemporalCase::new(false, [0.0, 0.0], 0.0, 1),
        trace_words,
    );

    let raw_left_variance = channel_variance(&raw_pixels, 0..2, 0);
    let filtered_left_variance = channel_variance(&filtered_lighting(&filtered), 0..2, 0);
    let raw_right_variance = channel_variance(&raw_pixels, 2..4, 2);
    let filtered_right_variance = channel_variance(&filtered_lighting(&filtered), 2..4, 2);
    assert!(
        filtered_left_variance < raw_left_variance * 0.45,
        "same-surface left radiance noise should be reduced: raw={raw_left_variance}, filtered={filtered_left_variance}"
    );
    assert!(
        filtered_right_variance < raw_right_variance * 0.45,
        "same-surface right radiance noise should be reduced: raw={raw_right_variance}, filtered={filtered_right_variance}"
    );

    let left_average = region_average(&filtered_lighting(&filtered), 0..2);
    let right_average = region_average(&filtered_lighting(&filtered), 2..4);
    assert!(
        left_average[0] > left_average[2] + 0.18,
        "left support surface should retain red dominance: {left_average:?}"
    );
    assert!(
        right_average[2] > right_average[0] + 0.18,
        "right support surface should retain blue dominance: {right_average:?}"
    );
}

#[test]
fn resolve_shader_declares_depth_normal_support_bilateral_spatial_filter() {
    let source =
        include_str!("../../../hybrid_gi/renderer/shaders/resolve_trace_depth_source.wgsl");

    assert!(source.contains("spatially_filtered_current_gi_sample"));
    assert!(source.contains("spatial_sample_is_compatible"));
    assert!(source.contains("SPATIAL_DEPTH_REJECTION_THRESHOLD"));
    assert!(source.contains("temporal_normal_matches(center.normal_code, candidate.normal_code)"));
    assert!(source.contains("candidate.source != center.source"));
    assert!(source.contains("abs(candidate.signature - center.signature)"));
}

#[test]
#[ignore]
fn export_depth_normal_support_spatial_denoise_wgpu_png() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping spatial denoise Wgpu product because no adapter is available");
        return;
    };
    let (trace_words, raw_pixels) = spatial_filter_trace_words();
    let filtered = run_temporal_resolve_pixels_with_trace_words(
        &device,
        &queue,
        TemporalCase::new(false, [0.0, 0.0], 0.0, 1),
        trace_words,
    );
    let filtered_pixels = filtered_lighting(&filtered);
    let raw_left_variance = channel_variance(&raw_pixels, 0..2, 0);
    let filtered_left_variance = channel_variance(&filtered_pixels, 0..2, 0);
    let raw_right_variance = channel_variance(&raw_pixels, 2..4, 2);
    let filtered_right_variance = channel_variance(&filtered_pixels, 2..4, 2);
    let left_average = region_average(&filtered_pixels, 0..2);
    let right_average = region_average(&filtered_pixels, 2..4);

    assert!(filtered_left_variance < raw_left_variance * 0.45);
    assert!(filtered_right_variance < raw_right_variance * 0.45);
    assert!(left_average[0] > left_average[2] + 0.18);
    assert!(right_average[2] > right_average[0] + 0.18);

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_spatial_filter_matrix_png(
        output_dir.join(SPATIAL_FILTER_PRODUCT_PNG),
        &raw_pixels,
        &filtered_pixels,
    );
    fs::write(
        output_dir.join(SPATIAL_FILTER_PRODUCT_REPORT),
        format!(
            "png={SPATIAL_FILTER_PRODUCT_PNG}\nleft=raw_trace_tile_radiance\nright=depth_normal_source_support_bilateral_filtered_wgpu_resolve\nwidth=257\nheight=128\ngpu_output_grid=4x4_resolve_pixels\nfilter_kernel=3x3_probe_grid_gaussian_1_2_1\nleft_support_signature={LEFT_SUPPORT_SIGNATURE}\nright_support_signature={RIGHT_SUPPORT_SIGNATURE}\nraw_left_red_variance={raw_left_variance:.6}\nfiltered_left_red_variance={filtered_left_variance:.6}\nraw_right_blue_variance={raw_right_variance:.6}\nfiltered_right_blue_variance={filtered_right_variance:.6}\nfiltered_left_average_rgb={:.6},{:.6},{:.6}\nfiltered_right_average_rgb={:.6},{:.6},{:.6}\nedge_contract=depth_plus_normal_dot_0.75_plus_source_plus_support_signature\ntemporal_order=spatial_filter_before_history_reprojection_and_accumulation\n",
            left_average[0],
            left_average[1],
            left_average[2],
            right_average[0],
            right_average[1],
            right_average[2],
        ),
    )
    .unwrap();
}

fn spatial_filter_trace_words() -> ([u32; TRACE_WORD_COUNT], Vec<[f32; 4]>) {
    let mut words = test_trace_words(
        SURFACE_CACHE_FLAG | RADIANCE_VALID_FLAG,
        [0; (TEST_SIZE * TEST_SIZE) as usize],
        [DEFAULT_NORMAL_CODE; (TEST_SIZE * TEST_SIZE) as usize],
    );
    for tile_y in 0..8_usize {
        for tile_x in 0..8_usize {
            let left_surface = tile_x < 4;
            set_trace_tile(
                &mut words,
                tile_x,
                tile_y,
                if left_surface {
                    [80, 50, 20, 255]
                } else {
                    [20, 50, 80, 255]
                },
                if left_surface {
                    LEFT_SUPPORT_SIGNATURE
                } else {
                    RIGHT_SUPPORT_SIGNATURE
                },
            );
        }
    }

    let mut raw_pixels = Vec::with_capacity((TEST_SIZE * TEST_SIZE) as usize);
    for pixel_y in 0..TEST_SIZE as usize {
        for pixel_x in 0..TEST_SIZE as usize {
            let left_surface = pixel_x < 2;
            let high_sample = (pixel_x + pixel_y) % 2 == 0;
            let rgba = match (left_surface, high_sample) {
                (true, true) => [200, 50, 20, 255],
                (true, false) => [20, 50, 20, 255],
                (false, true) => [20, 50, 200, 255],
                (false, false) => [20, 50, 20, 255],
            };
            set_trace_tile(
                &mut words,
                pixel_x * 2 + 1,
                pixel_y * 2 + 1,
                rgba,
                if left_surface {
                    LEFT_SUPPORT_SIGNATURE
                } else {
                    RIGHT_SUPPORT_SIGNATURE
                },
            );
            raw_pixels.push(rgba.map(|channel| f32::from(channel) / 255.0));
        }
    }
    (words, raw_pixels)
}

fn set_trace_tile(
    words: &mut [u32; TRACE_WORD_COUNT],
    tile_x: usize,
    tile_y: usize,
    radiance: [u8; 4],
    support_signature: u32,
) {
    let tile_index = tile_y * 8 + tile_x;
    let offset = TRACE_TILE_WORD_OFFSET + tile_index * TRACE_TILE_WORD_COUNT;
    words[offset] = pack_rgba8(radiance);
    words[offset + 6] = support_signature;
    words[offset + 7] = DEFAULT_NORMAL_CODE;
}

fn filtered_lighting(results: &[TemporalResult]) -> Vec<[f32; 4]> {
    results.iter().map(|result| result.lighting).collect()
}

fn channel_variance(pixels: &[[f32; 4]], x_range: std::ops::Range<usize>, channel: usize) -> f32 {
    let values = region_values(pixels, x_range, channel);
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    values
        .iter()
        .map(|value| (value - mean) * (value - mean))
        .sum::<f32>()
        / values.len() as f32
}

fn region_values(pixels: &[[f32; 4]], x_range: std::ops::Range<usize>, channel: usize) -> Vec<f32> {
    let mut values = Vec::new();
    for y in 0..TEST_SIZE as usize {
        for x in x_range.clone() {
            values.push(pixels[y * TEST_SIZE as usize + x][channel]);
        }
    }
    values
}

fn region_average(pixels: &[[f32; 4]], x_range: std::ops::Range<usize>) -> [f32; 3] {
    let mut sum = [0.0_f32; 3];
    let count = TEST_SIZE as usize * x_range.len();
    for y in 0..TEST_SIZE as usize {
        for x in x_range.clone() {
            for channel in 0..3 {
                sum[channel] += pixels[y * TEST_SIZE as usize + x][channel];
            }
        }
    }
    sum.map(|value| value / count as f32)
}

fn write_spatial_filter_matrix_png(
    path: PathBuf,
    raw_pixels: &[[f32; 4]],
    filtered_pixels: &[[f32; 4]],
) {
    const CELL_SIDE: u32 = 32;
    const PANEL_SIDE: u32 = TEST_SIZE * CELL_SIDE;
    let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(PANEL_SIDE * 2 + 1, PANEL_SIDE);
    for (panel_index, pixels) in [raw_pixels, filtered_pixels].into_iter().enumerate() {
        let panel_x = panel_index as u32 * (PANEL_SIDE + 1);
        for (pixel_index, color) in pixels.iter().enumerate() {
            let cell_x = pixel_index as u32 % TEST_SIZE;
            let cell_y = pixel_index as u32 / TEST_SIZE;
            let rgba = color.map(float_channel_to_u8);
            for y in 0..CELL_SIDE {
                for x in 0..CELL_SIDE {
                    image.put_pixel(
                        panel_x + cell_x * CELL_SIDE + x,
                        cell_y * CELL_SIDE + y,
                        if x == 0 || y == 0 {
                            Rgba([6, 8, 10, 255])
                        } else {
                            Rgba(rgba)
                        },
                    );
                }
            }
        }
    }
    for y in 0..PANEL_SIDE {
        image.put_pixel(PANEL_SIDE, y, Rgba([255, 255, 255, 255]));
    }
    image.save_with_format(path, ImageFormat::Png).unwrap();
}
