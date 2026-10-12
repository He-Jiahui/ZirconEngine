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
