use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::RenderHybridGiPreparedTraceRegionSceneData;
use crate::core::math::{Mat4, UVec2, Vec3};
use bytemuck::Zeroable;

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;
use super::super::super::super::hybrid_gi_trace_region_gpu::GpuHybridGiTraceRegion;
use super::super::camera_matrices::view_projection;

const HYBRID_GI_POSITION_BIAS: i32 = 2048;
const HYBRID_GI_POSITION_SCALE: f32 = 64.0;
const HYBRID_GI_TRACE_RADIUS_SCALE: f32 = 96.0;
const HYBRID_GI_TRACE_COVERAGE_SCALE: f32 = 128.0;

pub(in super::super) fn encode_hybrid_gi_trace_regions(
    frame: &ViewportRenderFrame,
    viewport_size: UVec2,
    enabled: bool,
) -> ([GpuHybridGiTraceRegion; MAX_HYBRID_GI_TRACE_REGIONS], u32) {
    let mut trace_regions = [GpuHybridGiTraceRegion::zeroed(); MAX_HYBRID_GI_TRACE_REGIONS];
    if !enabled {
        return (trace_regions, 0);
    }

    let Some(_hybrid_gi) = frame
        .extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .filter(|extract| extract.enabled)
    else {
        return (trace_regions, 0);
    };

    let Some(prepared_frame) = frame
        .prepared_runtime_sidebands()
        .hybrid_gi_prepared_frame()
    else {
        return (trace_regions, 0);
    };
    let scene_data_by_id = prepared_frame
        .trace_region_scene_data
        .iter()
        .map(|region| (region.region_id, region))
        .collect::<BTreeMap<_, _>>();
    let mut encoded_region_ids = BTreeSet::new();
    let camera = &frame.extract.view.camera;
    let view_proj = view_projection(camera, viewport_size);
    let camera_position = camera.transform.translation;
    let mut count = 0;

    for region_id in prepared_frame
        .scheduled_trace_region_ids
        .iter()
        .take(MAX_HYBRID_GI_TRACE_REGIONS)
    {
        if !encoded_region_ids.insert(*region_id) {
            continue;
        }
        let Some(region) = scene_data_by_id.get(region_id).copied() else {
            continue;
        };
        let Some(gpu_region) =
            project_prepared_hybrid_gi_trace_region(region, view_proj, camera_position)
        else {
            continue;
        };
        trace_regions[count] = gpu_region;
        count += 1;
    }

    (trace_regions, count as u32)
}

fn project_prepared_hybrid_gi_trace_region(
    region: &RenderHybridGiPreparedTraceRegionSceneData,
    view_proj: Mat4,
    camera_position: Vec3,
) -> Option<GpuHybridGiTraceRegion> {
    let bounds_center = Vec3::new(
        dequantized_signed(region.center_x_q),
        dequantized_signed(region.center_y_q),
        dequantized_signed(region.center_z_q),
    );
    let bounds_radius = region.radius_q as f32 / HYBRID_GI_TRACE_RADIUS_SCALE;
    let screen_coverage = region.coverage_q as f32 / HYBRID_GI_TRACE_COVERAGE_SCALE;
    let (uv_x, uv_y) = project_screen_uv(view_proj, bounds_center)?;
    let screen_radius = projected_screen_radius(bounds_radius, bounds_center, camera_position);
    let rt_lighting = [
        f32::from(region.rt_lighting_rgb[0]) / 255.0,
        f32::from(region.rt_lighting_rgb[1]) / 255.0,
        f32::from(region.rt_lighting_rgb[2]) / 255.0,
    ];

    Some(GpuHybridGiTraceRegion {
        screen_uv_and_radius: [uv_x, uv_y, screen_radius, 0.0],
        boost_and_coverage: [
            1.0,
            screen_coverage.clamp(0.0, 1.0),
            region.region_id as f32,
            0.0,
        ],
        rt_lighting_rgb_and_weight: [rt_lighting[0], rt_lighting[1], rt_lighting[2], 1.0],
    })
}

fn dequantized_signed(value: u32) -> f32 {
    (value as i32 - HYBRID_GI_POSITION_BIAS) as f32 / HYBRID_GI_POSITION_SCALE
}

fn project_screen_uv(view_proj: Mat4, position: Vec3) -> Option<(f32, f32)> {
    let clip = view_proj * position.extend(1.0);
    if clip.w.abs() <= f32::EPSILON {
        return None;
    }

    let ndc = clip.truncate() / clip.w;
    if ndc.z < -1.0 || ndc.z > 1.0 {
        return None;
    }

    Some((
        (0.5 + ndc.x * 0.5).clamp(0.0, 1.0),
        (0.5 - ndc.y * 0.5).clamp(0.0, 1.0),
    ))
}

fn projected_screen_radius(radius: f32, position: Vec3, camera_position: Vec3) -> f32 {
    let distance = (camera_position - position).length().max(1.0);
    (radius.max(0.05) / distance).clamp(0.04, 0.75)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiPreparedFrame,
        RenderHybridGiPreparedTraceRegionSceneData, RenderPreparedRuntimeSidebands,
    };
    use crate::core::math::UVec2;
    use crate::graphics::ViewportRenderFrame;
    use crate::scene::world::World;

    #[test]
    fn hybrid_gi_trace_region_encoder_returns_no_resources_when_disabled() {
        let frame = ViewportRenderFrame::from_extract(
            World::new().to_render_frame_extract(),
            UVec2::new(160, 120),
        );

        let (_, trace_region_count) =
            encode_hybrid_gi_trace_regions(&frame, UVec2::new(160, 120), false);

        assert_eq!(trace_region_count, 0);
    }

    #[test]
    fn hybrid_gi_trace_region_encoder_projects_prepared_scene_region_with_rt_lighting() {
        let frame = ViewportRenderFrame::from_extract(
            hybrid_gi_scene_representation_extract(),
            UVec2::new(160, 120),
        )
        .with_prepared_runtime_sidebands(
            RenderPreparedRuntimeSidebands::default().with_hybrid_gi_prepared_frame(Some(
                RenderHybridGiPreparedFrame {
                    scheduled_trace_region_ids: vec![300],
                    trace_region_scene_data: vec![RenderHybridGiPreparedTraceRegionSceneData {
                        region_id: 300,
                        center_x_q: 2048,
                        center_y_q: 2048,
                        center_z_q: 2048,
                        radius_q: 96,
                        coverage_q: 128,
                        rt_lighting_rgb: [255, 72, 48],
                    }],
                    ..RenderHybridGiPreparedFrame::default()
                },
            )),
        );

        let (trace_regions, trace_region_count) =
            encode_hybrid_gi_trace_regions(&frame, UVec2::new(160, 120), true);

        assert_eq!(trace_region_count, 1);
        assert!(trace_regions[0].screen_uv_and_radius[0] > 0.0);
        assert!(trace_regions[0].screen_uv_and_radius[2] > 0.0);
        assert_eq!(trace_regions[0].rt_lighting_rgb_and_weight[0], 1.0);
        assert!(trace_regions[0].rt_lighting_rgb_and_weight[3] > 0.0);
    }

    fn hybrid_gi_scene_representation_extract() -> RenderFrameExtract {
        let world = World::new();
        let mut extract = world.to_render_frame_extract();
        extract.apply_viewport_size(UVec2::new(160, 120));
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            trace_budget: 1,
            card_budget: 1,
            voxel_budget: 1,
            ..RenderHybridGiExtract::default()
        });
        extract
    }
}
