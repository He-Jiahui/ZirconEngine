use bytemuck::Zeroable;
use zircon_math::UVec2;

use crate::types::EditorOrRuntimeFrame;

use super::super::super::super::constants::MAX_HYBRID_GI_PROBES;
use super::super::super::super::hybrid_gi_probe_gpu::GpuHybridGiProbe;
use super::count_scheduled_trace_regions::count_scheduled_trace_regions;
use super::encode_hybrid_gi_probe_screen_data::encode_hybrid_gi_probe_screen_data;
use super::hybrid_gi_hierarchy_irradiance::hybrid_gi_hierarchy_irradiance;
use super::hybrid_gi_hierarchy_resolve_weight::hybrid_gi_hierarchy_resolve_weight;
use super::hybrid_gi_hierarchy_rt_lighting::hybrid_gi_hierarchy_rt_lighting;

pub(in super::super) fn encode_hybrid_gi_probes(
    frame: &EditorOrRuntimeFrame,
    viewport_size: UVec2,
    enabled: bool,
) -> ([GpuHybridGiProbe; MAX_HYBRID_GI_PROBES], u32, u32) {
    let mut probes = [GpuHybridGiProbe::zeroed(); MAX_HYBRID_GI_PROBES];
    if !enabled {
        return (probes, 0, 0);
    }

    let Some(prepare) = frame.hybrid_gi_prepare.as_ref() else {
        return (probes, 0, 0);
    };
    let hybrid_gi_extract = frame.extract.lighting.hybrid_global_illumination.as_ref();

    let mut count = 0;
    for probe in prepare.resident_probes.iter().take(MAX_HYBRID_GI_PROBES) {
        let (screen_data, hierarchy_weight, hierarchy_irradiance, hierarchy_rt_lighting) =
            hybrid_gi_extract
                .and_then(|extract| {
                    extract
                        .probes
                        .iter()
                        .find(|candidate| candidate.probe_id == probe.probe_id)
                })
                .map(|source| {
                    (
                        encode_hybrid_gi_probe_screen_data(&frame.extract, viewport_size, source),
                        hybrid_gi_hierarchy_resolve_weight(frame, source),
                        hybrid_gi_hierarchy_irradiance(frame, source),
                        hybrid_gi_hierarchy_rt_lighting(frame, source),
                    )
                })
                .unwrap_or(([0.5, 0.5, 1.0, 1.0], 1.0, [0.0; 4], [0.0; 4]));
        probes[count] = GpuHybridGiProbe {
            screen_uv_and_radius: screen_data,
            irradiance_and_intensity: [
                probe.irradiance_rgb[0] as f32 / 255.0,
                probe.irradiance_rgb[1] as f32 / 255.0,
                probe.irradiance_rgb[2] as f32 / 255.0,
                hierarchy_weight,
            ],
            hierarchy_irradiance_rgb_and_weight: hierarchy_irradiance,
            hierarchy_rt_lighting_rgb_and_weight: hierarchy_rt_lighting,
        };
        count += 1;
    }

    (probes, count as u32, count_scheduled_trace_regions(frame))
}
