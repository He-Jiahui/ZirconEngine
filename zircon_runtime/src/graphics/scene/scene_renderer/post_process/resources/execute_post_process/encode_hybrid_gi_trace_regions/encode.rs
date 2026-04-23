use crate::core::math::UVec2;
use bytemuck::Zeroable;

use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;
use super::super::super::super::hybrid_gi_trace_region_gpu::GpuHybridGiTraceRegion;
use super::encode_hybrid_gi_trace_region_screen_data::encode_hybrid_gi_trace_region_screen_data;
use super::hybrid_gi_trace_region_intensity::hybrid_gi_trace_region_intensity;
use super::hybrid_gi_trace_region_rt_lighting::hybrid_gi_trace_region_rt_lighting;

pub(in super::super) fn encode_hybrid_gi_trace_regions(
    frame: &ViewportRenderFrame,
    viewport_size: UVec2,
    enabled: bool,
) -> ([GpuHybridGiTraceRegion; MAX_HYBRID_GI_TRACE_REGIONS], u32) {
    let mut trace_regions = [GpuHybridGiTraceRegion::zeroed(); MAX_HYBRID_GI_TRACE_REGIONS];
    if !enabled {
        return (trace_regions, 0);
    }
    if frame.hybrid_gi_scene_prepare.is_some() {
        return (trace_regions, 0);
    }

    let Some(prepare) = frame.hybrid_gi_prepare.as_ref() else {
        return (trace_regions, 0);
    };
    let Some(hybrid_gi_extract) = frame.extract.lighting.hybrid_global_illumination.as_ref() else {
        return (trace_regions, 0);
    };

    let mut count = 0;
    for region_id in prepare
        .scheduled_trace_region_ids
        .iter()
        .take(MAX_HYBRID_GI_TRACE_REGIONS)
    {
        let Some(region) = hybrid_gi_extract
            .trace_regions
            .iter()
            .find(|candidate| candidate.region_id == *region_id)
        else {
            continue;
        };
        trace_regions[count] = GpuHybridGiTraceRegion {
            screen_uv_and_radius: encode_hybrid_gi_trace_region_screen_data(
                &frame.extract,
                viewport_size,
                region,
            ),
            boost_and_coverage: [
                hybrid_gi_trace_region_intensity(region, hybrid_gi_extract.tracing_budget),
                region.screen_coverage.clamp(0.1, 1.0),
                0.0,
                0.0,
            ],
            rt_lighting_rgb_and_weight: hybrid_gi_trace_region_rt_lighting(region),
        };
        count += 1;
    }

    (trace_regions, count as u32)
}
