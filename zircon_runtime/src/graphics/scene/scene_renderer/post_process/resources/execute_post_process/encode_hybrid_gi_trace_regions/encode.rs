use crate::core::framework::render::RenderHybridGiTraceRegion;
use crate::core::math::{Mat4, UVec2, Vec3};
use bytemuck::Zeroable;

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;
use super::super::super::super::hybrid_gi_trace_region_gpu::GpuHybridGiTraceRegion;
use super::super::camera_matrices::view_projection;

pub(in super::super) fn encode_hybrid_gi_trace_regions(
    frame: &ViewportRenderFrame,
    viewport_size: UVec2,
    enabled: bool,
) -> ([GpuHybridGiTraceRegion; MAX_HYBRID_GI_TRACE_REGIONS], u32) {
    let mut trace_regions = [GpuHybridGiTraceRegion::zeroed(); MAX_HYBRID_GI_TRACE_REGIONS];
    if !enabled {
        return (trace_regions, 0);
    }

    let Some(hybrid_gi) = frame
        .extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .filter(|extract| extract.enabled)
    else {
        return (trace_regions, 0);
    };

    let camera = &frame.extract.view.camera;
    let view_proj = view_projection(camera, viewport_size);
    let camera_position = camera.transform.translation;
    let mut count = 0;

    for region in hybrid_gi
        .trace_regions
        .iter()
        .take(MAX_HYBRID_GI_TRACE_REGIONS)
    {
        let Some(gpu_region) = project_hybrid_gi_trace_region(region, view_proj, camera_position)
        else {
            continue;
        };
        trace_regions[count] = gpu_region;
        count += 1;
    }

    (trace_regions, count as u32)
}

fn project_hybrid_gi_trace_region(
    region: &RenderHybridGiTraceRegion,
    view_proj: Mat4,
    camera_position: Vec3,
) -> Option<GpuHybridGiTraceRegion> {
    let (uv_x, uv_y) = project_screen_uv(view_proj, region.bounds_center)?;
    let screen_radius =
        projected_screen_radius(region.bounds_radius, region.bounds_center, camera_position);
    let rt_lighting = [
        f32::from(region.rt_lighting_rgb[0]) / 255.0,
        f32::from(region.rt_lighting_rgb[1]) / 255.0,
        f32::from(region.rt_lighting_rgb[2]) / 255.0,
    ];

    Some(GpuHybridGiTraceRegion {
        screen_uv_and_radius: [uv_x, uv_y, screen_radius, 0.0],
        boost_and_coverage: [
            1.0,
            region.screen_coverage.clamp(0.0, 1.0),
            region.region_id as f32,
            0.0,
        ],
        rt_lighting_rgb_and_weight: [rt_lighting[0], rt_lighting[1], rt_lighting[2], 1.0],
    })
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
    use crate::core::framework::render::{RenderFrameExtract, RenderHybridGiExtract};
    use crate::core::math::{UVec2, Vec3};
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
    fn hybrid_gi_trace_region_encoder_projects_visible_region_with_rt_lighting() {
        let frame = ViewportRenderFrame::from_extract(
            hybrid_gi_extract_with_trace_region(),
            UVec2::new(160, 120),
        );

        let (trace_regions, trace_region_count) =
            encode_hybrid_gi_trace_regions(&frame, UVec2::new(160, 120), true);

        assert_eq!(trace_region_count, 1);
        assert!(trace_regions[0].screen_uv_and_radius[0] > 0.0);
        assert!(trace_regions[0].screen_uv_and_radius[2] > 0.0);
        assert_eq!(trace_regions[0].rt_lighting_rgb_and_weight[0], 1.0);
        assert!(trace_regions[0].rt_lighting_rgb_and_weight[3] > 0.0);
    }

    fn hybrid_gi_extract_with_trace_region() -> RenderFrameExtract {
        let world = World::new();
        let mesh = world
            .nodes()
            .iter()
            .find(|node| node.mesh.is_some())
            .map(|node| node.id)
            .expect("default world should contain a renderable mesh");
        let mut extract = world.to_render_frame_extract();
        extract.apply_viewport_size(UVec2::new(160, 120));
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            trace_regions: vec![RenderHybridGiTraceRegion {
                entity: mesh,
                region_id: 300,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_coverage: 1.0,
                rt_lighting_rgb: [255, 72, 48],
            }],
            ..RenderHybridGiExtract::default()
        });
        extract
    }
}
