use crate::hybrid_gi::types::HybridGiScenePrepareFrame;
use zircon_runtime::core::framework::render::{ProjectionMode, RenderFrameExtract};
use zircon_runtime::core::math::{UVec2, Vec3};

use super::super::camera_matrices::view_projection;
use super::hybrid_gi_budget_weight::hybrid_gi_budget_weight;
use super::hybrid_gi_probe_source::HybridGiProbeSource;

pub(super) fn encode_hybrid_gi_probe_screen_data<S: HybridGiProbeSource + ?Sized>(
    extract: &RenderFrameExtract,
    viewport_size: UVec2,
    probe: &S,
) -> [f32; 4] {
    encode_hybrid_gi_bounds_screen_data(
        extract,
        viewport_size,
        probe.position(),
        probe.radius(),
        hybrid_gi_budget_weight(probe.ray_budget()),
    )
}

pub(super) fn encode_hybrid_gi_scene_driven_probe_screen_data(
    extract: &RenderFrameExtract,
    viewport_size: UVec2,
    scene_prepare: &HybridGiScenePrepareFrame,
    ray_budget: u32,
) -> [f32; 4] {
    let budget_weight = hybrid_gi_budget_weight(ray_budget);
    let Some((bounds_center, bounds_radius)) =
        scene_prepare_aggregate_bounds_center_and_radius(scene_prepare)
    else {
        return [0.5, 0.5, 1.0, budget_weight];
    };

    encode_hybrid_gi_bounds_screen_data(
        extract,
        viewport_size,
        bounds_center,
        bounds_radius,
        budget_weight,
    )
}

pub(super) fn encode_hybrid_gi_scene_truth_fallback_probe_screen_data(ray_budget: u32) -> [f32; 4] {
    [0.5, 0.5, 1.0, hybrid_gi_budget_weight(ray_budget)]
}

fn encode_hybrid_gi_bounds_screen_data(
    extract: &RenderFrameExtract,
    viewport_size: UVec2,
    bounds_center: Vec3,
    bounds_radius: f32,
    budget_weight: f32,
) -> [f32; 4] {
    let camera = &extract.view.camera;
    let (view, projection) = view_projection(camera, viewport_size);
    let clip = projection * view * bounds_center.extend(1.0);
    if clip.w.abs() <= f32::EPSILON {
        return [0.5, 0.5, 1.0, budget_weight];
    }

    let ndc = clip.truncate() / clip.w;
    let uv_x = (0.5 + ndc.x * 0.5).clamp(0.0, 1.0);
    let uv_y = (0.5 - ndc.y * 0.5).clamp(0.0, 1.0);
    let view_position = view.transform_point3(bounds_center);
    let radius = match camera.projection_mode {
        ProjectionMode::Perspective => {
            let distance = (-view_position.z).max(1.0);
            (bounds_radius.max(0.05) / distance).clamp(0.06, 0.85)
        }
        ProjectionMode::Orthographic => {
            let half_height = camera.ortho_size.max(0.01);
            (bounds_radius.max(0.05) / (half_height * 2.0)).clamp(0.04, 0.85)
        }
    };

    [uv_x, uv_y, radius, budget_weight]
}

fn scene_prepare_aggregate_bounds_center_and_radius(
    scene_prepare: &HybridGiScenePrepareFrame,
) -> Option<(Vec3, f32)> {
    scene_prepare_surface_bounds_center_and_radius(scene_prepare)
        .or_else(|| scene_prepare_voxel_bounds_center_and_radius(scene_prepare))
}

fn scene_prepare_surface_bounds_center_and_radius(
    scene_prepare: &HybridGiScenePrepareFrame,
) -> Option<(Vec3, f32)> {
    let mut bounds_min = None::<Vec3>;
    let mut bounds_max = None::<Vec3>;

    for request in &scene_prepare.card_capture_requests {
        extend_aggregate_bounds(
            &mut bounds_min,
            &mut bounds_max,
            request.bounds_center,
            request.bounds_radius,
        );
    }
    for page_content in &scene_prepare.surface_cache_page_contents {
        extend_aggregate_bounds(
            &mut bounds_min,
            &mut bounds_max,
            page_content.bounds_center,
            page_content.bounds_radius,
        );
    }

    aggregate_bounds_center_and_radius(bounds_min, bounds_max)
}

fn scene_prepare_voxel_bounds_center_and_radius(
    scene_prepare: &HybridGiScenePrepareFrame,
) -> Option<(Vec3, f32)> {
    let mut bounds_min = None::<Vec3>;
    let mut bounds_max = None::<Vec3>;

    for clipmap in &scene_prepare.voxel_clipmaps {
        extend_aggregate_bounds(
            &mut bounds_min,
            &mut bounds_max,
            clipmap.center,
            clipmap.half_extent,
        );
    }

    aggregate_bounds_center_and_radius(bounds_min, bounds_max)
}

fn aggregate_bounds_center_and_radius(
    bounds_min: Option<Vec3>,
    bounds_max: Option<Vec3>,
) -> Option<(Vec3, f32)> {
    let (bounds_min, bounds_max) = bounds_min.zip(bounds_max)?;
    let center = (bounds_min + bounds_max) * 0.5;
    let radius = (bounds_max - center).length().max(0.05);
    Some((center, radius))
}

fn extend_aggregate_bounds(
    bounds_min: &mut Option<Vec3>,
    bounds_max: &mut Option<Vec3>,
    center: Vec3,
    radius: f32,
) {
    let radius_vec = Vec3::splat(radius.max(0.05));
    let entry_min = center - radius_vec;
    let entry_max = center + radius_vec;
    *bounds_min = Some(match *bounds_min {
        Some(current_min) => current_min.min(entry_min),
        None => entry_min,
    });
    *bounds_max = Some(match *bounds_max {
        Some(current_max) => current_max.max(entry_max),
        None => entry_max,
    });
}
