use zircon_framework::render::RenderHybridGiTraceRegion;

pub(super) fn hybrid_gi_trace_region_rt_lighting(region: &RenderHybridGiTraceRegion) -> [f32; 4] {
    let rgb = [
        region.rt_lighting_rgb[0] as f32 / 255.0,
        region.rt_lighting_rgb[1] as f32 / 255.0,
        region.rt_lighting_rgb[2] as f32 / 255.0,
    ];
    let max_component = rgb[0].max(rgb[1]).max(rgb[2]);

    [rgb[0], rgb[1], rgb[2], max_component]
}
