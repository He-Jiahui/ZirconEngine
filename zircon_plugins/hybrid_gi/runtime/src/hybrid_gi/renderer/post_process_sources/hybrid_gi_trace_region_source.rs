use std::collections::BTreeMap;

use crate::hybrid_gi::types::HybridGiResolveTraceRegionSceneData;
use zircon_runtime::core::framework::render::RenderHybridGiExtract;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderHybridGiTraceRegion;
use zircon_runtime::core::math::Vec3;
use zircon_runtime::graphics::hybrid_gi_extract_sources::{
    enabled_hybrid_gi_extract, hybrid_gi_extract_trace_region_records_by_id,
    hybrid_gi_extract_uses_scene_representation_budget, HybridGiExtractTraceRegionRecord,
};

const TRACE_REGION_POSITION_SCALE: f32 = 64.0;
const TRACE_REGION_POSITION_BIAS: i32 = 2048;
const TRACE_REGION_RADIUS_SCALE: f32 = 96.0;
const TRACE_REGION_COVERAGE_SCALE: f32 = 128.0;

pub(super) trait HybridGiTraceRegionSource {
    fn bounds_center(&self) -> Vec3;
    fn bounds_radius(&self) -> f32;
    fn screen_coverage(&self) -> f32;
    fn rt_lighting_rgb(&self) -> [u8; 3];
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct HybridGiExtractTraceRegionSource {
    bounds_center: Vec3,
    bounds_radius: f32,
    screen_coverage: f32,
    rt_lighting_rgb: [u8; 3],
}

pub(super) fn fallback_trace_region_sources_by_id(
    extract: Option<&RenderHybridGiExtract>,
) -> BTreeMap<u32, HybridGiExtractTraceRegionSource> {
    let Some(extract) = enabled_hybrid_gi_extract(extract) else {
        return BTreeMap::new();
    };
    if hybrid_gi_extract_uses_scene_representation_budget(extract) {
        return BTreeMap::new();
    }

    hybrid_gi_extract_trace_region_records_by_id(extract)
        .into_iter()
        .map(|(region_id, region)| (region_id, HybridGiExtractTraceRegionSource::from(region)))
        .collect()
}

impl From<HybridGiExtractTraceRegionRecord> for HybridGiExtractTraceRegionSource {
    fn from(region: HybridGiExtractTraceRegionRecord) -> Self {
        Self {
            bounds_center: region.bounds_center,
            bounds_radius: region.bounds_radius,
            screen_coverage: region.screen_coverage,
            rt_lighting_rgb: region.rt_lighting_rgb,
        }
    }
}

impl HybridGiTraceRegionSource for HybridGiExtractTraceRegionSource {
    fn bounds_center(&self) -> Vec3 {
        self.bounds_center
    }

    fn bounds_radius(&self) -> f32 {
        self.bounds_radius
    }

    fn screen_coverage(&self) -> f32 {
        self.screen_coverage
    }

    fn rt_lighting_rgb(&self) -> [u8; 3] {
        self.rt_lighting_rgb
    }
}

#[cfg(test)]
impl HybridGiTraceRegionSource for RenderHybridGiTraceRegion {
    fn bounds_center(&self) -> Vec3 {
        self.bounds_center
    }

    fn bounds_radius(&self) -> f32 {
        self.bounds_radius
    }

    fn screen_coverage(&self) -> f32 {
        self.screen_coverage
    }

    fn rt_lighting_rgb(&self) -> [u8; 3] {
        self.rt_lighting_rgb
    }
}

impl HybridGiTraceRegionSource for HybridGiResolveTraceRegionSceneData {
    fn bounds_center(&self) -> Vec3 {
        Vec3::new(
            dequantized_signed(self.center_x_q()),
            dequantized_signed(self.center_y_q()),
            dequantized_signed(self.center_z_q()),
        )
    }

    fn bounds_radius(&self) -> f32 {
        self.radius_q() as f32 / TRACE_REGION_RADIUS_SCALE
    }

    fn screen_coverage(&self) -> f32 {
        self.coverage_q() as f32 / TRACE_REGION_COVERAGE_SCALE
    }

    fn rt_lighting_rgb(&self) -> [u8; 3] {
        self.rt_lighting_rgb()
    }
}

fn dequantized_signed(value: u32) -> f32 {
    (value as i32 - TRACE_REGION_POSITION_BIAS) as f32 / TRACE_REGION_POSITION_SCALE
}
