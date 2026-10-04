use super::*;

#[test]
fn prepared_runtime_sidebands_report_empty_only_without_payloads() {
    assert!(RenderPreparedRuntimeSidebands::default().is_empty());
    assert!(RenderHybridGiPreparedFrame::default().is_empty());
    assert!(!RenderHybridGiPreparedFrame {
        probe_rt_lighting_rgb: vec![RenderHybridGiPreparedProbeRtLighting {
            probe_id: 3,
            rt_lighting_rgb: [8, 16, 24],
        }],
        ..RenderHybridGiPreparedFrame::default()
    }
    .is_empty());
    assert!(!RenderHybridGiPreparedFrame {
        radiance_cache_instance_id: 9,
        radiance_cache_updates: vec![RenderHybridGiPreparedRadianceCacheUpdate {
            slot: 3,
            generation: 7,
            radiance_rgb: [8, 16, 24],
            confidence_q8: 200,
            reuse_committed_radiance: false,
        }],
        ..RenderHybridGiPreparedFrame::default()
    }
    .is_empty());
    assert!(!RenderHybridGiPreparedFrame {
        radiance_cache_instance_id: 9,
        radiance_cache_bootstrap_updates: vec![RenderHybridGiPreparedRadianceCacheUpdate {
            slot: 3,
            generation: 7,
            radiance_rgb: [8, 16, 24],
            confidence_q8: 200,
            reuse_committed_radiance: false,
        }],
        ..RenderHybridGiPreparedFrame::default()
    }
    .is_empty());
    assert!(!RenderHybridGiPreparedFrame {
        composite_policy: RenderHybridGiCompositePolicy::baked_baseline_with_dynamic_delta(7, 3),
        ..RenderHybridGiPreparedFrame::default()
    }
    .is_empty());

    let sidebands = RenderPreparedRuntimeSidebands::new(
        RenderPluginRendererOutputs::default(),
        vec![5],
        Vec::new(),
    );

    assert!(!sidebands.is_empty());
    assert_eq!(sidebands.hybrid_gi_evictable_probe_ids(), &[5]);
}

#[test]
fn hybrid_gi_composite_policy_rejects_full_dynamic_baked_double_ownership() {
    let full_dynamic = RenderHybridGiCompositePolicy::full_dynamic(4);
    assert!(full_dynamic.accepts_hybrid_gi_output());
    assert_eq!(full_dynamic.source_mask(), HYBRID_GI_SOURCE_FULL_DYNAMIC);

    let baked_delta = RenderHybridGiCompositePolicy::baked_baseline_with_dynamic_delta(9, 5);
    assert!(baked_delta.accepts_hybrid_gi_output());
    assert_eq!(baked_delta.baked_light_set_generation(), Some(9));
    assert_eq!(baked_delta.participation_epoch(), 5);
    assert_eq!(
        baked_delta.source_mask(),
        HYBRID_GI_SOURCE_BAKED_BASELINE | HYBRID_GI_SOURCE_DYNAMIC_DELTA
    );
    assert_eq!(
        baked_delta.source_mask() & HYBRID_GI_SOURCE_FULL_DYNAMIC,
        0,
        "a baked baseline must never share a lobe with full-dynamic indirect light"
    );
}
