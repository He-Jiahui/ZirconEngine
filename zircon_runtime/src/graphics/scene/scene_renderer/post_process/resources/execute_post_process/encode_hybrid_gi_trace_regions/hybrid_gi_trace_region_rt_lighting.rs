use super::super::hybrid_gi_trace_region_source::HybridGiTraceRegionSource;

pub(super) fn hybrid_gi_trace_region_rt_lighting<S: HybridGiTraceRegionSource + ?Sized>(
    region: &S,
) -> [f32; 4] {
    hybrid_gi_trace_region_rt_lighting_from_rgb(region.rt_lighting_rgb())
}

pub(super) fn hybrid_gi_trace_region_rt_lighting_from_rgb(rt_lighting_rgb: [u8; 3]) -> [f32; 4] {
    let rgb = [
        rt_lighting_rgb[0] as f32 / 255.0,
        rt_lighting_rgb[1] as f32 / 255.0,
        rt_lighting_rgb[2] as f32 / 255.0,
    ];
    let max_component = rgb[0].max(rgb[1]).max(rgb[2]);

    [rgb[0], rgb[1], rgb[2], max_component]
}
