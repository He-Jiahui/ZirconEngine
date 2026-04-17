use zircon_math::UVec2;
use zircon_scene::{ProjectionMode, RenderFrameExtract, RenderHybridGiProbe};

use super::super::camera_matrices::view_projection;
use super::hybrid_gi_budget_weight::hybrid_gi_budget_weight;

pub(super) fn encode_hybrid_gi_probe_screen_data(
    extract: &RenderFrameExtract,
    viewport_size: UVec2,
    probe: &RenderHybridGiProbe,
) -> [f32; 4] {
    let camera = &extract.view.camera;
    let (view, projection) = view_projection(camera, viewport_size);
    let clip = projection * view * probe.position.extend(1.0);
    if clip.w.abs() <= f32::EPSILON {
        return [0.5, 0.5, 1.0, hybrid_gi_budget_weight(probe.ray_budget)];
    }

    let ndc = clip.truncate() / clip.w;
    let uv_x = (0.5 + ndc.x * 0.5).clamp(0.0, 1.0);
    let uv_y = (0.5 - ndc.y * 0.5).clamp(0.0, 1.0);
    let view_position = view.transform_point3(probe.position);
    let radius = match camera.projection_mode {
        ProjectionMode::Perspective => {
            let distance = (-view_position.z).max(1.0);
            (probe.radius.max(0.05) / distance).clamp(0.06, 0.65)
        }
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            (probe.radius.max(0.05) / (half_height * 2.0)).clamp(0.04, 0.65)
        }
    };

    [
        uv_x,
        uv_y,
        radius,
        hybrid_gi_budget_weight(probe.ray_budget),
    ]
}
