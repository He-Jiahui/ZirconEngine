use crate::core::framework::render::{ProjectionMode, RenderFrameExtract};
use crate::core::math::{UVec2, Vec3};
use crate::graphics::types::HybridGiResolveTraceRegionSceneData;

use super::super::camera_matrices::view_projection;
use super::super::hybrid_gi_trace_region_source::HybridGiTraceRegionSource;

pub(super) fn encode_hybrid_gi_trace_region_screen_data<S: HybridGiTraceRegionSource + ?Sized>(
    extract: &RenderFrameExtract,
    viewport_size: UVec2,
    region: &S,
) -> [f32; 4] {
    encode_trace_region_screen_data(
        extract,
        viewport_size,
        region.bounds_center(),
        region.bounds_radius(),
        region.screen_coverage(),
    )
}

pub(super) fn encode_hybrid_gi_runtime_trace_region_screen_data(
    extract: &RenderFrameExtract,
    viewport_size: UVec2,
    region: HybridGiResolveTraceRegionSceneData,
) -> [f32; 4] {
    encode_hybrid_gi_trace_region_screen_data(extract, viewport_size, &region)
}

fn encode_trace_region_screen_data(
    extract: &RenderFrameExtract,
    viewport_size: UVec2,
    bounds_center: Vec3,
    bounds_radius: f32,
    screen_coverage: f32,
) -> [f32; 4] {
    let camera = &extract.view.camera;
    let (view, projection) = view_projection(camera, viewport_size);
    let clip = projection * view * bounds_center.extend(1.0);
    if clip.w.abs() <= f32::EPSILON {
        return [0.5, 0.5, 1.0, 1.0];
    }

    let ndc = clip.truncate() / clip.w;
    let uv_x = (0.5 + ndc.x * 0.5).clamp(0.0, 1.0);
    let uv_y = (0.5 - ndc.y * 0.5).clamp(0.0, 1.0);
    let view_position = view.transform_point3(bounds_center);
    let projected_radius = match camera.projection_mode {
        ProjectionMode::Perspective => {
            let distance = (-view_position.z).max(1.0);
            (bounds_radius.max(0.05) / distance).clamp(0.08, 0.8)
        }
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            (bounds_radius.max(0.05) / (half_height * 2.0)).clamp(0.06, 0.8)
        }
    };
    let coverage_radius =
        (projected_radius * (0.65 + screen_coverage.clamp(0.0, 1.0))).clamp(0.08, 0.95);

    [uv_x, uv_y, coverage_radius, 1.0]
}

pub(super) fn dequantized_trace_region_coverage(
    region: HybridGiResolveTraceRegionSceneData,
) -> f32 {
    region.screen_coverage()
}
