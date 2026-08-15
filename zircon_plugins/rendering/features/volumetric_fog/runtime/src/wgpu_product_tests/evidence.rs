use std::path::PathBuf;

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_runtime::core::framework::render::{CapturedFrame, RenderStats};

use super::{PNG_NAME, VIEWPORT_SIZE};

#[derive(Clone, Copy, Debug)]
pub(super) struct FrameComparisonMetrics {
    pub(super) changed_pixels: usize,
    pub(super) brighter_pixels: usize,
    pub(super) darker_pixels: usize,
    pub(super) color_shifted_pixels: usize,
    pub(super) rgb_abs_delta: u64,
    pub(super) baseline_visible_pixels: usize,
    pub(super) volumetric_visible_pixels: usize,
    pub(super) baseline_max_luma: f32,
    pub(super) volumetric_max_luma: f32,
    pub(super) window_light_shaft_sample_pixels: usize,
    pub(super) window_light_shaft_brighter_pixels: usize,
    pub(super) window_light_shaft_average_luma_delta: f32,
    pub(super) shadow_control_sample_pixels: usize,
    pub(super) shadow_control_average_luma_delta: f32,
}

pub(super) fn volumetric_product_gate_passed(metrics: FrameComparisonMetrics) -> bool {
    volumetric_scattering_gate_passed(metrics)
        && metrics.window_light_shaft_sample_pixels > 0
        && metrics.shadow_control_sample_pixels > 0
        && metrics.window_light_shaft_brighter_pixels * 5 > metrics.window_light_shaft_sample_pixels
        && metrics.window_light_shaft_average_luma_delta > 1.5
        && metrics.window_light_shaft_average_luma_delta
            > metrics.shadow_control_average_luma_delta + 1.5
}

pub(super) fn volumetric_scattering_gate_passed(metrics: FrameComparisonMetrics) -> bool {
    metrics.changed_pixels > 1_000
        && metrics.brighter_pixels > 350
        && metrics.color_shifted_pixels > 350
        && metrics.rgb_abs_delta > 18_000
}

pub(super) fn compare_frames(
    baseline: &CapturedFrame,
    volumetric: &CapturedFrame,
) -> FrameComparisonMetrics {
    assert_eq!(
        (baseline.width, baseline.height),
        (volumetric.width, volumetric.height)
    );
    let mut metrics = FrameComparisonMetrics {
        changed_pixels: 0,
        brighter_pixels: 0,
        darker_pixels: 0,
        color_shifted_pixels: 0,
        rgb_abs_delta: 0,
        baseline_visible_pixels: 0,
        volumetric_visible_pixels: 0,
        baseline_max_luma: 0.0,
        volumetric_max_luma: 0.0,
        window_light_shaft_sample_pixels: 0,
        window_light_shaft_brighter_pixels: 0,
        window_light_shaft_average_luma_delta: 0.0,
        shadow_control_sample_pixels: 0,
        shadow_control_average_luma_delta: 0.0,
    };
    let mut window_light_shaft_luma_delta = 0.0;
    let mut shadow_control_luma_delta = 0.0;
    for (pixel_index, (baseline_pixel, volumetric_pixel)) in baseline
        .rgba
        .chunks_exact(4)
        .zip(volumetric.rgba.chunks_exact(4))
        .enumerate()
    {
        let baseline_luma = rgb_luma(baseline_pixel);
        let volumetric_luma = rgb_luma(volumetric_pixel);
        let luma_delta = volumetric_luma - baseline_luma;
        let x = pixel_index as u32 % baseline.width;
        let y = pixel_index as u32 / baseline.width;
        let pixel_delta = baseline_pixel[..3]
            .iter()
            .zip(&volumetric_pixel[..3])
            .map(|(lhs, rhs)| (*lhs as i16 - *rhs as i16).unsigned_abs() as u64)
            .sum::<u64>();
        metrics.rgb_abs_delta += pixel_delta;
        metrics.changed_pixels += usize::from(pixel_delta > 3);
        metrics.brighter_pixels += usize::from(volumetric_luma > baseline_luma + 1.5);
        metrics.darker_pixels += usize::from(baseline_luma > volumetric_luma + 1.5);
        metrics.color_shifted_pixels +=
            usize::from(chromaticity_distance(baseline_pixel, volumetric_pixel) > 0.035);
        metrics.baseline_visible_pixels += usize::from(baseline_luma > 4.0);
        metrics.volumetric_visible_pixels += usize::from(volumetric_luma > 4.0);
        metrics.baseline_max_luma = metrics.baseline_max_luma.max(baseline_luma);
        metrics.volumetric_max_luma = metrics.volumetric_max_luma.max(volumetric_luma);
        if pixel_is_in_window_light_shaft(x, y, baseline.width, baseline.height) {
            metrics.window_light_shaft_sample_pixels += 1;
            metrics.window_light_shaft_brighter_pixels += usize::from(luma_delta > 1.5);
            window_light_shaft_luma_delta += luma_delta;
        }
        if pixel_is_in_shadow_control(x, y, baseline.width, baseline.height) {
            metrics.shadow_control_sample_pixels += 1;
            shadow_control_luma_delta += luma_delta;
        }
    }
    if metrics.window_light_shaft_sample_pixels > 0 {
        metrics.window_light_shaft_average_luma_delta =
            window_light_shaft_luma_delta / metrics.window_light_shaft_sample_pixels as f32;
    }
    if metrics.shadow_control_sample_pixels > 0 {
        metrics.shadow_control_average_luma_delta =
            shadow_control_luma_delta / metrics.shadow_control_sample_pixels as f32;
    }
    metrics
}

pub(super) fn write_side_by_side_png(path: PathBuf, left: &CapturedFrame, right: &CapturedFrame) {
    assert_eq!(left.height, right.height);
    let output_width = left.width + 1 + right.width;
    let mut rgba = vec![0u8; (output_width * left.height * 4) as usize];
    for y in 0..left.height {
        let output_row = (y * output_width * 4) as usize;
        let left_row = (y * left.width * 4) as usize;
        let left_len = (left.width * 4) as usize;
        rgba[output_row..output_row + left_len]
            .copy_from_slice(&left.rgba[left_row..left_row + left_len]);

        let separator = output_row + left_len;
        rgba[separator..separator + 4].copy_from_slice(&[255, 255, 255, 255]);

        let right_row = (y * right.width * 4) as usize;
        let right_len = (right.width * 4) as usize;
        let right_start = separator + 4;
        rgba[right_start..right_start + right_len]
            .copy_from_slice(&right.rgba[right_row..right_row + right_len]);
    }
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(output_width, left.height, rgba)
        .expect("side-by-side volumetric product payload");
    image
        .save_with_format(path, ImageFormat::Png)
        .expect("volumetric product PNG should be writable");
}

pub(super) fn format_report(
    baseline_stats: &RenderStats,
    volumetric_stats: &RenderStats,
    unshadowed_metrics: FrameComparisonMetrics,
    metrics: FrameComparisonMetrics,
    product_gate_passed: bool,
) -> String {
    let status = if product_gate_passed {
        "render_plan18_af_m3_volumetric_compiled_scene_window_light_shaft_perf_wgpu_passed"
    } else {
        "render_plan18_af_m3_volumetric_compiled_scene_window_light_shaft_perf_wgpu_diagnostic_failed"
    };
    format!(
        concat!(
            "status={}\n",
            "artifact={}\n",
            "layout=baseline_left|one_pixel_separator|volumetric_high_temporal_right\n",
            "viewport={}x{}\n",
            "reference=dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricFog.cpp\n",
            "scene=shadowed_directional_light_through_window_frame_into_foggy_room\n",
            "unshadowed_changed_pixels={}\n",
            "unshadowed_brighter_pixels={}\n",
            "unshadowed_color_shifted_pixels={}\n",
            "unshadowed_rgb_abs_delta={}\n",
            "baseline_visibility_inputs={}\n",
            "baseline_visibility_frustum_culled={}\n",
            "baseline_visibility_visible={}\n",
            "baseline_materials={}\n",
            "baseline_materials_ready={}\n",
            "baseline_material_fallbacks={}\n",
            "baseline_mesh_draws={}\n",
            "baseline_mesh_opaque_draws={}\n",
            "baseline_mesh_commands={}\n",
            "baseline_volumetric_dispatches={}\n",
            "baseline_volumetric_dispatch_groups={}\n",
            "baseline_volumetric_uploaded_bytes={}\n",
            "volumetric_visibility_inputs={}\n",
            "volumetric_visibility_frustum_culled={}\n",
            "volumetric_visibility_visible={}\n",
            "volumetric_materials={}\n",
            "volumetric_materials_ready={}\n",
            "volumetric_material_fallbacks={}\n",
            "volumetric_mesh_draws={}\n",
            "volumetric_mesh_opaque_draws={}\n",
            "volumetric_mesh_commands={}\n",
            "volumetric_dispatches={}\n",
            "volumetric_dispatch_groups={}\n",
            "volumetric_uploaded_bytes={}\n",
            "matched_compute_workloads={}\n",
            "missing_compute_dispatches={}\n",
            "mismatched_compute_workloads={}\n",
            "unexpected_compute_dispatches={}\n",
            "light_grid_lights={}\n",
            "light_grid_non_empty_clusters={}\n",
            "light_grid_peak_lights_per_cluster={}\n",
            "shadow_pass_count={}\n",
            "shadow_atlas_write_count={}\n",
            "shadow_caster_draw_count={}\n",
            "shadow_directional_light_ready_count={}\n",
            "changed_pixels={}\n",
            "brighter_pixels={}\n",
            "darker_pixels={}\n",
            "color_shifted_pixels={}\n",
            "rgb_abs_delta={}\n",
            "baseline_visible_pixels={}\n",
            "volumetric_visible_pixels={}\n",
            "baseline_max_luma={:.3}\n",
            "volumetric_max_luma={:.3}\n",
            "window_light_shaft_sample_pixels={}\n",
            "window_light_shaft_brighter_pixels={}\n",
            "window_light_shaft_average_luma_delta={:.3}\n",
            "shadow_control_sample_pixels={}\n",
            "shadow_control_average_luma_delta={:.3}\n",
            "window_light_shaft_shadow_contrast_luma={:.3}\n",
        ),
        status,
        PNG_NAME,
        VIEWPORT_SIZE.x,
        VIEWPORT_SIZE.y,
        unshadowed_metrics.changed_pixels,
        unshadowed_metrics.brighter_pixels,
        unshadowed_metrics.color_shifted_pixels,
        unshadowed_metrics.rgb_abs_delta,
        baseline_stats.last_visibility_input_count,
        baseline_stats.last_visibility_frustum_culled_count,
        baseline_stats.last_visibility_visible_count,
        baseline_stats.last_material_count,
        baseline_stats.last_material_ready_count,
        baseline_stats.last_material_fallback_count,
        baseline_stats.last_mesh_draw_count,
        baseline_stats.last_mesh_opaque_draw_count,
        baseline_stats.last_mesh_command_count,
        baseline_stats.last_volumetric_fog_compute_dispatch_count,
        baseline_stats.last_volumetric_fog_compute_dispatch_group_count,
        baseline_stats.last_volumetric_fog_uploaded_bytes,
        volumetric_stats.last_visibility_input_count,
        volumetric_stats.last_visibility_frustum_culled_count,
        volumetric_stats.last_visibility_visible_count,
        volumetric_stats.last_material_count,
        volumetric_stats.last_material_ready_count,
        volumetric_stats.last_material_fallback_count,
        volumetric_stats.last_mesh_draw_count,
        volumetric_stats.last_mesh_opaque_draw_count,
        volumetric_stats.last_mesh_command_count,
        volumetric_stats.last_volumetric_fog_compute_dispatch_count,
        volumetric_stats.last_volumetric_fog_compute_dispatch_group_count,
        volumetric_stats.last_volumetric_fog_uploaded_bytes,
        volumetric_stats.last_graph_compute_matched_workload_count,
        volumetric_stats.last_graph_compute_missing_dispatch_count,
        volumetric_stats.last_graph_compute_workload_mismatch_count,
        volumetric_stats.last_graph_compute_unexpected_dispatch_count,
        volumetric_stats.last_light_grid_light_count,
        volumetric_stats.last_light_grid_non_empty_cluster_count,
        volumetric_stats.last_light_grid_peak_lights_per_cluster,
        volumetric_stats
            .last_shadow_execution_report
            .shadow_pass_count,
        volumetric_stats
            .last_shadow_execution_report
            .shadow_atlas_write_count,
        volumetric_stats
            .last_shadow_execution_report
            .caster_draw_count,
        volumetric_stats
            .last_shadow_execution_report
            .directional_light_ready_count,
        metrics.changed_pixels,
        metrics.brighter_pixels,
        metrics.darker_pixels,
        metrics.color_shifted_pixels,
        metrics.rgb_abs_delta,
        metrics.baseline_visible_pixels,
        metrics.volumetric_visible_pixels,
        metrics.baseline_max_luma,
        metrics.volumetric_max_luma,
        metrics.window_light_shaft_sample_pixels,
        metrics.window_light_shaft_brighter_pixels,
        metrics.window_light_shaft_average_luma_delta,
        metrics.shadow_control_sample_pixels,
        metrics.shadow_control_average_luma_delta,
        metrics.window_light_shaft_average_luma_delta - metrics.shadow_control_average_luma_delta,
    )
}

pub(super) fn render_output_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(5)
        .expect("volumetric plugin manifest should be nested under repository root")
        .join("docs/tests/runtime/render")
}

fn comparison_frame(rgba: Vec<u8>) -> CapturedFrame {
    CapturedFrame::new(2, 1, rgba, 0)
}

fn synthetic_comparison_frame(
    width: u32,
    height: u32,
    pixel: impl Fn(u32, u32) -> [u8; 4],
) -> CapturedFrame {
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            rgba.extend_from_slice(&pixel(x, y));
        }
    }
    CapturedFrame::new(width, height, rgba, 0)
}

fn pixel_is_in_window_light_shaft(x: u32, y: u32, width: u32, height: u32) -> bool {
    let normalized_x = (x as f32 + 0.5) / width as f32;
    let normalized_y = (y as f32 + 0.5) / height as f32;
    if !(0.55..=0.95).contains(&normalized_y) {
        return false;
    }
    let shaft_progress = (normalized_y - 0.55) / 0.40;
    let shaft_half_width = 0.08 + shaft_progress * 0.28;
    (normalized_x - 0.5).abs() <= shaft_half_width
}

fn pixel_is_in_shadow_control(x: u32, y: u32, width: u32, height: u32) -> bool {
    let normalized_x = (x as f32 + 0.5) / width as f32;
    let normalized_y = (y as f32 + 0.5) / height as f32;
    (0.55..=0.80).contains(&normalized_y) && (normalized_x <= 0.20 || normalized_x >= 0.80)
}

fn rgb_luma(pixel: &[u8]) -> f32 {
    pixel[0] as f32 * 0.2126 + pixel[1] as f32 * 0.7152 + pixel[2] as f32 * 0.0722
}

fn chromaticity_distance(lhs: &[u8], rhs: &[u8]) -> f32 {
    let lhs_sum = lhs[..3].iter().map(|value| *value as f32).sum::<f32>();
    let rhs_sum = rhs[..3].iter().map(|value| *value as f32).sum::<f32>();
    if lhs_sum <= 1.0 || rhs_sum <= 1.0 {
        return 0.0;
    }
    lhs[..3]
        .iter()
        .zip(&rhs[..3])
        .map(|(lhs, rhs)| (*lhs as f32 / lhs_sum - *rhs as f32 / rhs_sum).abs())
        .sum()
}

#[test]
fn volumetric_product_metrics_distinguish_extinction_from_colored_in_scatter() {
    let baseline = comparison_frame(vec![120, 120, 120, 255, 120, 120, 120, 255]);
    let volumetric = comparison_frame(vec![90, 90, 90, 255, 115, 78, 54, 255]);

    let metrics = compare_frames(&baseline, &volumetric);

    assert_eq!(metrics.darker_pixels, 2);
    assert_eq!(metrics.color_shifted_pixels, 1);
}

#[test]
fn volumetric_product_gate_accepts_spatially_concentrated_window_light_shaft() {
    let baseline = synthetic_comparison_frame(64, 32, |_, _| [20, 20, 20, 255]);
    let volumetric = synthetic_comparison_frame(64, 32, |x, y| {
        if pixel_is_in_window_light_shaft(x, y, 64, 32) {
            [65, 45, 25, 255]
        } else {
            [26, 22, 20, 255]
        }
    });

    let metrics = compare_frames(&baseline, &volumetric);

    assert!(metrics.window_light_shaft_average_luma_delta > 10.0);
    assert!(
        metrics.window_light_shaft_average_luma_delta
            > metrics.shadow_control_average_luma_delta + 1.5
    );
    assert!(volumetric_product_gate_passed(metrics));
}

#[test]
fn volumetric_product_gate_rejects_uniform_full_frame_fog_change() {
    let baseline = synthetic_comparison_frame(64, 32, |_, _| [20, 20, 20, 255]);
    let volumetric = synthetic_comparison_frame(64, 32, |_, _| [60, 42, 24, 255]);

    let metrics = compare_frames(&baseline, &volumetric);

    assert!(metrics.changed_pixels > 1_000);
    assert!(metrics.color_shifted_pixels > 350);
    assert!(
        (metrics.window_light_shaft_average_luma_delta - metrics.shadow_control_average_luma_delta)
            .abs()
            < 0.01
    );
    assert!(!volumetric_product_gate_passed(metrics));
}

#[test]
fn volumetric_product_report_exposes_visibility_material_and_mesh_diagnostics() {
    let metrics = compare_frames(
        &comparison_frame(vec![0, 0, 0, 255, 0, 0, 0, 255]),
        &comparison_frame(vec![1, 2, 3, 255, 1, 2, 3, 255]),
    );
    let report = format_report(
        &RenderStats::default(),
        &RenderStats::default(),
        metrics,
        metrics,
        false,
    );

    for field in [
        "baseline_visibility_inputs=0",
        "baseline_materials_ready=0",
        "baseline_mesh_draws=0",
        "volumetric_visibility_inputs=0",
        "volumetric_materials_ready=0",
        "volumetric_mesh_draws=0",
    ] {
        assert!(report.contains(field), "missing report field `{field}`");
    }
}
