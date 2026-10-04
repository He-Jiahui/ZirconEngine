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
