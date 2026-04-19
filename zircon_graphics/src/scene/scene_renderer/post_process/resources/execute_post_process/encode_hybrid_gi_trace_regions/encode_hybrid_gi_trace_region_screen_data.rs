use zircon_framework::render::{ProjectionMode, RenderFrameExtract, RenderHybridGiTraceRegion};
use zircon_math::UVec2;

use super::super::camera_matrices::view_projection;

pub(super) fn encode_hybrid_gi_trace_region_screen_data(
    extract: &RenderFrameExtract,
    viewport_size: UVec2,
    region: &RenderHybridGiTraceRegion,
) -> [f32; 4] {
    let camera = &extract.view.camera;
    let (view, projection) = view_projection(camera, viewport_size);
    let clip = projection * view * region.bounds_center.extend(1.0);
    if clip.w.abs() <= f32::EPSILON {
        return [0.5, 0.5, 1.0, 1.0];
    }

    let ndc = clip.truncate() / clip.w;
    let uv_x = (0.5 + ndc.x * 0.5).clamp(0.0, 1.0);
    let uv_y = (0.5 - ndc.y * 0.5).clamp(0.0, 1.0);
    let view_position = view.transform_point3(region.bounds_center);
    let projected_radius = match camera.projection_mode {
        ProjectionMode::Perspective => {
            let distance = (-view_position.z).max(1.0);
            (region.bounds_radius.max(0.05) / distance).clamp(0.08, 0.8)
        }
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            (region.bounds_radius.max(0.05) / (half_height * 2.0)).clamp(0.06, 0.8)
        }
    };
    let coverage_radius =
        (projected_radius * (0.65 + region.screen_coverage.clamp(0.0, 1.0))).clamp(0.08, 0.95);

    [uv_x, uv_y, coverage_radius, 1.0]
}
