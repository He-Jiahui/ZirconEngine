use zircon_runtime::core::framework::render::{
    RenderHybridGiProbeTraceDiagnosticRecord, RenderHybridGiTraceCostCounters,
    RenderHybridGiTraceFallbackReason, RenderHybridGiTraceIntersectionSource,
    RenderHybridGiTraceLightingSource, RENDER_HYBRID_GI_PROBE_TRACE_DIAGNOSTIC_WORD_COUNT,
};
use zircon_runtime::graphics::GraphicsError;

use super::read_buffer_u32s::read_buffer_u32s;

pub(in crate::hybrid_gi::renderer::gpu_readback) fn probe_trace_diagnostics(
    bytes: &[u8],
    word_count: usize,
) -> Result<Vec<RenderHybridGiProbeTraceDiagnosticRecord>, GraphicsError> {
    let words = read_buffer_u32s(bytes, word_count)?;
    let record_count = words.first().copied().unwrap_or_default() as usize;
    Ok(words
        .get(1..)
        .unwrap_or_default()
        .chunks_exact(RENDER_HYBRID_GI_PROBE_TRACE_DIAGNOSTIC_WORD_COUNT)
        .take(record_count)
        .map(decode_record)
        .collect())
}

fn decode_record(words: &[u32]) -> RenderHybridGiProbeTraceDiagnosticRecord {
    RenderHybridGiProbeTraceDiagnosticRecord {
        probe_id: words[0],
        intersection_source: match words[1] {
            1 => RenderHybridGiTraceIntersectionSource::SurfaceCache,
            2 => RenderHybridGiTraceIntersectionSource::GlobalSdf,
            3 => RenderHybridGiTraceIntersectionSource::VoxelClipmap,
            4 => RenderHybridGiTraceIntersectionSource::HardwareRayTracing,
            _ => RenderHybridGiTraceIntersectionSource::Miss,
        },
        lighting_source: match words[2] {
            1 => RenderHybridGiTraceLightingSource::SurfaceCache,
            2 => RenderHybridGiTraceLightingSource::ProbeLineage,
            3 => RenderHybridGiTraceLightingSource::VoxelRadiance,
            _ => RenderHybridGiTraceLightingSource::NeutralAmbient,
        },
        intersection_backend_mask: words[3],
        lighting_source_mask: words[4],
        distance_bits: words[5],
        confidence_bits: words[6],
        fallback_reason: match words[7] {
            1 => RenderHybridGiTraceFallbackReason::ScreenDataUnavailable,
            2 => RenderHybridGiTraceFallbackReason::HardwareRayTracingUnavailable,
            3 => RenderHybridGiTraceFallbackReason::GlobalSdfUnavailable,
            4 => RenderHybridGiTraceFallbackReason::IntersectionMiss,
            5 => RenderHybridGiTraceFallbackReason::LightingUnavailable,
            _ => RenderHybridGiTraceFallbackReason::None,
        },
        cost: RenderHybridGiTraceCostCounters {
            texture_samples: words[8],
            page_tests: words[9],
            sdf_steps: words[10],
            voxel_candidates: words[11],
            hardware_rays: words[12],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_bounded_trace_provenance_and_cost_records() {
        let words = [
            1_u32,
            17,
            2,
            2,
            6,
            12,
            3.5_f32.to_bits(),
            0.75_f32.to_bits(),
            1,
            4,
            8,
            6,
            3,
            0,
        ];

        let records = probe_trace_diagnostics(bytemuck::cast_slice(&words), words.len()).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].probe_id, 17);
        assert_eq!(
            records[0].intersection_source,
            RenderHybridGiTraceIntersectionSource::GlobalSdf
        );
        assert_eq!(records[0].intersection_backend_mask, 6);
        assert_eq!(records[0].lighting_source_mask, 12);
        assert_eq!(
            records[0].lighting_source,
            RenderHybridGiTraceLightingSource::ProbeLineage
        );
        assert_eq!(
            records[0].fallback_reason,
            RenderHybridGiTraceFallbackReason::ScreenDataUnavailable
        );
        assert_eq!(records[0].cost.page_tests, 8);
        assert_eq!(records[0].cost.sdf_steps, 6);
    }
}
