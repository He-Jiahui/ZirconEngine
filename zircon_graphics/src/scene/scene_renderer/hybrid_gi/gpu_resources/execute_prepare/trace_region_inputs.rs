use zircon_scene::RenderHybridGiExtract;

use crate::types::HybridGiPrepareFrame;

use super::super::gpu_trace_region_input::GpuTraceRegionInput;
use super::super::seed_quantization::{quantized_positive, quantized_signed};

pub(super) fn trace_region_inputs(
    prepare: &HybridGiPrepareFrame,
    extract: Option<&RenderHybridGiExtract>,
) -> Vec<GpuTraceRegionInput> {
    prepare
        .scheduled_trace_region_ids
        .iter()
        .filter_map(|region_id| {
            extract
                .and_then(|extract| {
                    extract
                        .trace_regions
                        .iter()
                        .find(|region| region.region_id == *region_id)
                })
                .map(|region| GpuTraceRegionInput {
                    region_id: region.region_id,
                    center_x_q: quantized_signed(region.bounds_center.x),
                    center_y_q: quantized_signed(region.bounds_center.y),
                    center_z_q: quantized_signed(region.bounds_center.z),
                    radius_q: quantized_positive(region.bounds_radius, 96.0),
                    coverage_q: quantized_positive(region.screen_coverage, 128.0),
                    rt_lighting_rgb: pack_rgb8(region.rt_lighting_rgb),
                    _padding1: 0,
                })
        })
        .collect()
}

fn pack_rgb8(rgb: [u8; 3]) -> u32 {
    u32::from(rgb[0]) | (u32::from(rgb[1]) << 8) | (u32::from(rgb[2]) << 16)
}
