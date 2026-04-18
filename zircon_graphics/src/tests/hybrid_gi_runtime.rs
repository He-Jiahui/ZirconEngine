use zircon_scene::{RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion};

use crate::{
    runtime::{HybridGiProbeResidencyState, HybridGiProbeUpdateRequest, HybridGiRuntimeState},
    types::{HybridGiPrepareFrame, HybridGiPrepareProbe, HybridGiPrepareUpdateRequest},
    VisibilityHybridGiFeedback, VisibilityHybridGiUpdatePlan,
};

#[test]
fn hybrid_gi_runtime_state_tracks_cache_residency_pending_updates_and_trace_schedule() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe(200, true, 64),
            probe(300, false, 128),
            probe(500, true, 32),
        ],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(state.probe_slot(500), Some(1));
    assert_eq!(state.probe_slot(300), None);
    assert_eq!(
        state.probe_residency(200),
        Some(HybridGiProbeResidencyState::Resident)
    );
    assert_eq!(
        state.probe_residency(500),
        Some(HybridGiProbeResidencyState::Resident)
    );
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::PendingUpdate)
    );
    assert_eq!(
        state.pending_updates(),
        vec![HybridGiProbeUpdateRequest {
            probe_id: 300,
            ray_budget: 128,
            generation: 9,
        }]
    );
    assert_eq!(state.scheduled_trace_regions(), vec![40]);
    assert_eq!(state.evictable_probes(), vec![500]);

    let snapshot = state.snapshot();
    assert_eq!(snapshot.cache_entry_count, 2);
    assert_eq!(snapshot.resident_probe_count, 2);
    assert_eq!(snapshot.pending_update_count, 1);
    assert_eq!(snapshot.scheduled_trace_region_count, 1);
}

#[test]
fn hybrid_gi_runtime_state_deduplicates_probe_updates_and_reuses_evicted_slots() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe(200, true, 64),
            probe(300, false, 128),
            probe(500, true, 32),
        ],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );
    state.ingest_plan(
        10,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![50],
            evictable_probe_ids: vec![500],
        },
    );

    assert_eq!(state.pending_updates().len(), 1);
    state.apply_evictions([500]);
    state.fulfill_updates([300]);

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(state.probe_slot(300), Some(1));
    assert_eq!(state.probe_slot(500), None);
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::Resident)
    );
    assert_eq!(
        state.pending_updates(),
        Vec::<HybridGiProbeUpdateRequest>::new()
    );
    assert_eq!(state.scheduled_trace_regions(), vec![50]);

    let snapshot = state.snapshot();
    assert_eq!(snapshot.cache_entry_count, 2);
    assert_eq!(snapshot.resident_probe_count, 2);
    assert_eq!(snapshot.pending_update_count, 0);
    assert_eq!(snapshot.scheduled_trace_region_count, 1);
}

#[test]
fn hybrid_gi_runtime_state_builds_prepare_frame_without_host_bootstrap_irradiance() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 2,
        probes: vec![
            probe(200, true, 64),
            probe(300, false, 128),
            probe(500, true, 32),
        ],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );

    assert_eq!(
        state.build_prepare_frame(),
        HybridGiPrepareFrame {
            resident_probes: vec![
                HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 64,
                    irradiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareProbe {
                    probe_id: 500,
                    slot: 1,
                    ray_budget: 32,
                    irradiance_rgb: [0, 0, 0],
                },
            ],
            pending_updates: vec![HybridGiPrepareUpdateRequest {
                probe_id: 300,
                ray_budget: 128,
                generation: 9,
            }],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        }
    );
}

#[test]
fn hybrid_gi_runtime_state_consumes_feedback_and_promotes_requested_probes() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe(200, true, 64),
            probe(300, false, 128),
            probe(500, true, 32),
        ],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );

    state.consume_feedback(&VisibilityHybridGiFeedback {
        active_probe_ids: vec![300, 200],
        requested_probe_ids: vec![300],
        scheduled_trace_region_ids: vec![50],
        evictable_probe_ids: vec![500],
    });

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(state.probe_slot(300), Some(1));
    assert_eq!(state.probe_slot(500), None);
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::Resident)
    );
    assert_eq!(
        state.pending_updates(),
        Vec::<HybridGiProbeUpdateRequest>::new()
    );
    assert_eq!(state.scheduled_trace_regions(), vec![50]);

    let snapshot = state.snapshot();
    assert_eq!(snapshot.cache_entry_count, 2);
    assert_eq!(snapshot.resident_probe_count, 2);
    assert_eq!(snapshot.pending_update_count, 0);
    assert_eq!(snapshot.scheduled_trace_region_count, 1);
}

#[test]
fn hybrid_gi_runtime_state_leaves_updates_pending_without_evictable_budget() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![probe(200, true, 64), probe(300, false, 128)],
        trace_regions: vec![trace_region(40), trace_region(60)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    state.consume_feedback(&VisibilityHybridGiFeedback {
        active_probe_ids: vec![300, 200],
        requested_probe_ids: vec![300],
        scheduled_trace_region_ids: vec![60],
        evictable_probe_ids: Vec::new(),
    });

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(state.probe_slot(300), None);
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::PendingUpdate)
    );
    assert_eq!(
        state.pending_updates(),
        vec![HybridGiProbeUpdateRequest {
            probe_id: 300,
            ray_budget: 128,
            generation: 9,
        }]
    );
    assert_eq!(state.scheduled_trace_regions(), vec![60]);
}

#[test]
fn hybrid_gi_runtime_state_applies_gpu_completed_updates_and_trace_schedule() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe(200, true, 64),
            probe(300, false, 128),
            probe(500, true, 32),
        ],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );

    state.complete_gpu_updates(
        [300],
        [50],
        &[
            (200, [121, 133, 145]),
            (300, [210, 164, 118]),
            (500, [89, 101, 113]),
        ],
        &[
            (200, [24, 32, 40]),
            (300, [176, 88, 48]),
            (500, [16, 20, 24]),
        ],
        &[500],
    );

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(state.probe_slot(300), Some(1));
    assert_eq!(state.probe_slot(500), None);
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::Resident)
    );
    assert_eq!(
        state.pending_updates(),
        Vec::<HybridGiProbeUpdateRequest>::new()
    );
    assert_eq!(state.scheduled_trace_regions(), vec![50]);
    assert_eq!(
        state.build_prepare_frame(),
        HybridGiPrepareFrame {
            resident_probes: vec![
                HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 64,
                    irradiance_rgb: [121, 133, 145],
                },
                HybridGiPrepareProbe {
                    probe_id: 300,
                    slot: 1,
                    ray_budget: 128,
                    irradiance_rgb: [210, 164, 118],
                },
            ],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![50],
            evictable_probe_ids: Vec::new(),
        }
    );
}

#[test]
fn hybrid_gi_runtime_state_applies_gpu_cache_snapshot_as_residency_truth() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        probe_budget: 3,
        tracing_budget: 1,
        probes: vec![
            probe(200, true, 64),
            probe(300, false, 128),
            probe(500, true, 32),
            probe(600, false, 48),
        ],
        trace_regions: vec![trace_region(40), trace_region(50)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        9,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300, 600],
            dirty_requested_probe_ids: vec![300, 600],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );

    state.apply_gpu_cache_entries(&[(200, 0), (300, 1)]);

    assert_eq!(state.probe_slot(200), Some(0));
    assert_eq!(state.probe_slot(300), Some(1));
    assert_eq!(state.probe_slot(500), None);
    assert_eq!(state.probe_slot(600), None);
    assert_eq!(
        state.probe_residency(300),
        Some(HybridGiProbeResidencyState::Resident)
    );
    assert_eq!(
        state.probe_residency(600),
        Some(HybridGiProbeResidencyState::PendingUpdate)
    );
    assert_eq!(
        state.pending_updates(),
        vec![HybridGiProbeUpdateRequest {
            probe_id: 600,
            ray_budget: 48,
            generation: 9,
        }]
    );
    assert!(state.evictable_probes().is_empty());

    assert_eq!(
        state.build_prepare_frame(),
        HybridGiPrepareFrame {
            resident_probes: vec![
                HybridGiPrepareProbe {
                    probe_id: 200,
                    slot: 0,
                    ray_budget: 64,
                    irradiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareProbe {
                    probe_id: 300,
                    slot: 1,
                    ray_budget: 128,
                    irradiance_rgb: [0, 0, 0],
                },
            ],
            pending_updates: vec![HybridGiPrepareUpdateRequest {
                probe_id: 600,
                ray_budget: 48,
                generation: 9,
            }],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        }
    );
}

#[test]
fn hybrid_gi_runtime_state_drops_stale_scene_probes_and_pending_updates_when_extract_shrinks() {
    let mut state = HybridGiRuntimeState::default();
    let initial_extract = RenderHybridGiExtract {
        probe_budget: 3,
        tracing_budget: 1,
        probes: vec![
            probe(100, true, 96),
            probe(200, false, 64),
            probe(300, true, 48),
        ],
        trace_regions: vec![trace_region(40)],
    };

    state.register_extract(Some(&initial_extract));
    state.ingest_plan(
        12,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 300],
            requested_probe_ids: vec![200],
            dirty_requested_probe_ids: vec![200],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![300],
        },
    );
    state.complete_gpu_updates(
        [200],
        [40],
        &[
            (100, [192, 144, 96]),
            (200, [96, 144, 192]),
            (300, [48, 64, 80]),
        ],
        &[
            (100, [208, 96, 48]),
            (200, [128, 144, 160]),
            (300, [24, 32, 40]),
        ],
        &[300],
    );

    let shrunk_extract = RenderHybridGiExtract {
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![probe(100, true, 96)],
        trace_regions: vec![trace_region(50)],
    };

    state.register_extract(Some(&shrunk_extract));

    assert_eq!(state.probe_slot(100), Some(0));
    assert_eq!(state.probe_slot(200), None);
    assert_eq!(state.probe_slot(300), None);
    assert_eq!(
        state.probe_residency(200),
        None,
        "expected runtime host state to purge removed scene probes instead of keeping stale pending/resident entries alive"
    );
    assert_eq!(
        state.probe_residency(300),
        None,
        "expected runtime host state to evict removed resident probes when the extract no longer contains their lineage"
    );
    assert_eq!(
        state.pending_updates(),
        Vec::<HybridGiProbeUpdateRequest>::new(),
        "expected removed scene probes to drop out of the pending update queue"
    );
    assert_eq!(state.scheduled_trace_regions(), Vec::<u32>::new());
    assert_eq!(state.evictable_probes(), Vec::<u32>::new());
    assert_eq!(
        state.build_prepare_frame(),
        HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: 100,
                slot: 0,
                ray_budget: 96,
                irradiance_rgb: [192, 144, 96],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: Vec::new(),
            evictable_probe_ids: Vec::new(),
        }
    );
}

#[test]
fn hybrid_gi_runtime_state_withholds_descendant_probe_updates_while_ancestor_update_remains_pending(
) {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe(100, true, 96),
            probe_with_parent(200, false, 64, 100),
            probe_with_parent(300, false, 48, 200),
        ],
        trace_regions: vec![trace_region(40)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        21,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![200, 300],
            dirty_requested_probe_ids: vec![200, 300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let blocked_prepare = state.build_prepare_frame();
    assert_eq!(
        blocked_prepare.pending_updates,
        vec![HybridGiPrepareUpdateRequest {
            probe_id: 200,
            ray_budget: 64,
            generation: 21,
        }],
        "expected runtime prepare to withhold descendant probe updates while the missing ancestor probe update is still pending so hierarchy-aware GPU completion does not bypass the collapsed lineage"
    );

    state.complete_gpu_updates(
        [200],
        [40],
        &[(100, [48, 48, 48]), (200, [96, 128, 160])],
        &[(100, [160, 96, 48]), (200, [128, 144, 160])],
        &[],
    );

    let unblocked_prepare = state.build_prepare_frame();
    assert_eq!(
        unblocked_prepare.pending_updates,
        vec![HybridGiPrepareUpdateRequest {
            probe_id: 300,
            ray_budget: 48,
            generation: 21,
        }],
        "expected descendant probe updates to re-enter the prepare queue once their pending ancestor probe becomes resident"
    );
}

#[test]
fn hybrid_gi_runtime_state_prioritizes_pending_ancestor_probes_that_reconnect_hot_descendants() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        probe_budget: 3,
        tracing_budget: 1,
        probes: vec![
            probe(100, true, 96),
            probe_with_parent(200, false, 72, 100),
            probe_with_parent(400, true, 56, 200),
            probe(800, false, 48),
        ],
        trace_regions: vec![trace_region(40)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        31,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 400],
            requested_probe_ids: vec![800, 200],
            dirty_requested_probe_ids: vec![800, 200],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let prepare = state.build_prepare_frame();
    assert_eq!(
        prepare.pending_updates,
        vec![
            HybridGiPrepareUpdateRequest {
                probe_id: 200,
                ray_budget: 72,
                generation: 31,
            },
            HybridGiPrepareUpdateRequest {
                probe_id: 800,
                ray_budget: 48,
                generation: 31,
            },
        ],
        "expected runtime prepare to prioritize the missing ancestor probe that reconnects already-resident descendant history before unrelated pending probe updates so the hierarchy-aware radiance-cache path converges instead of thrashing hot descendants"
    );
}

#[test]
fn hybrid_gi_runtime_state_builds_resolve_runtime_from_gpu_trace_lighting_history() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe(200, true, 96),
            probe(300, false, 64),
            probe(500, true, 48),
        ],
        trace_regions: vec![trace_region(40)],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        13,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![200, 500],
            requested_probe_ids: vec![300],
            dirty_requested_probe_ids: vec![300],
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: vec![500],
        },
    );
    state.complete_gpu_updates(
        [300],
        [40],
        &[
            (200, [112, 128, 144]),
            (300, [160, 144, 128]),
            (500, [72, 88, 104]),
        ],
        &[
            (200, [208, 96, 48]),
            (300, [176, 104, 64]),
            (500, [32, 48, 80]),
        ],
        &[500],
    );

    assert_eq!(
        state.build_resolve_runtime().probe_rt_lighting_rgb,
        std::collections::BTreeMap::from([(200, [208, 96, 48]), (300, [176, 104, 64])]),
        "expected runtime-host resolve inputs to retain GPU-produced per-probe trace-lighting truth for resident probes so post-process resolve can consume GPU source instead of recomputing all RT tint encode-side"
    );
}

#[test]
fn hybrid_gi_runtime_state_builds_hierarchy_resolve_runtime_from_resident_lineage_history() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        probe_budget: 4,
        tracing_budget: 1,
        probes: vec![
            probe(100, true, 96),
            probe_with_parent(200, true, 80, 100),
            probe_with_parent(300, true, 64, 200),
            probe_with_parent(400, true, 48, 300),
        ],
        trace_regions: vec![trace_region(40)],
    };

    state.register_extract(Some(&extract));
    state.complete_gpu_updates(
        [100, 200, 300, 400],
        [40],
        &[
            (100, [220, 180, 120]),
            (200, [180, 144, 112]),
            (300, [144, 112, 88]),
            (400, [96, 96, 96]),
        ],
        &[
            (100, [240, 80, 32]),
            (200, [208, 112, 48]),
            (300, [176, 96, 64]),
            (400, [96, 96, 96]),
        ],
        &[],
    );

    let runtime = state.build_resolve_runtime();
    assert!(
        runtime
            .hierarchy_resolve_weight(400)
            .is_some_and(|weight| weight > 1.4),
        "expected runtime-host resolve inputs to carry hierarchy-aware resolve weight for deeper resident probe lineages instead of leaving that weighting exclusively to encode-time hierarchy scans"
    );
    assert!(
        runtime
            .hierarchy_irradiance(400)
            .is_some_and(|encoded| encoded[3] > 0.1 && encoded[0] > encoded[2]),
        "expected runtime-host resolve inputs to carry farther-ancestor irradiance continuation for deeper resident probe lineages instead of recomputing it only from current-frame prepare ancestry"
    );
    assert!(
        runtime
            .hierarchy_rt_lighting(400)
            .is_some_and(|encoded| encoded[3] > 0.1 && encoded[0] > encoded[2]),
        "expected runtime-host resolve inputs to carry ancestor-derived RT-lighting continuation for deeper resident probe lineages instead of leaving that continuation exclusively to encode-time hierarchy scans"
    );
}

#[test]
fn hybrid_gi_runtime_state_prioritizes_pending_probe_with_stronger_lineage_trace_support() {
    let mut state = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        probe_budget: 3,
        tracing_budget: 2,
        probes: vec![
            probe_at(100, true, 96, zircon_math::Vec3::new(-0.9, 0.0, 0.0)),
            probe_with_parent_at(200, false, 72, 100, zircon_math::Vec3::new(0.0, 0.0, 0.0)),
            probe_at(300, false, 80, zircon_math::Vec3::new(0.55, 0.0, 0.0)),
        ],
        trace_regions: vec![
            trace_region_at(40, zircon_math::Vec3::ZERO),
            trace_region_at(50, zircon_math::Vec3::new(-0.9, 0.0, 0.0)),
        ],
    };

    state.register_extract(Some(&extract));
    state.ingest_plan(
        56,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100],
            requested_probe_ids: vec![300, 200],
            dirty_requested_probe_ids: vec![300, 200],
            scheduled_trace_region_ids: vec![40, 50],
            evictable_probe_ids: Vec::new(),
        },
    );

    let prepare = state.build_prepare_frame();
    assert_eq!(
        prepare.pending_updates,
        vec![
            HybridGiPrepareUpdateRequest {
                probe_id: 200,
                ray_budget: 72,
                generation: 56,
            },
            HybridGiPrepareUpdateRequest {
                probe_id: 300,
                ray_budget: 80,
                generation: 56,
            },
        ],
        "expected runtime prepare to prioritize the pending probe whose nonresident lineage stays aligned with the scheduled trace hierarchy instead of only sorting by flat descendant counts or shallow depth"
    );
}

#[test]
fn hybrid_gi_runtime_state_strengthens_resolve_weight_when_trace_schedule_supports_lineage() {
    let hierarchical_extract = RenderHybridGiExtract {
        probe_budget: 2,
        tracing_budget: 1,
        probes: vec![
            probe_at(100, true, 96, zircon_math::Vec3::new(-0.8, 0.0, 0.0)),
            probe_with_parent_at(200, true, 96, 100, zircon_math::Vec3::ZERO),
        ],
        trace_regions: vec![trace_region_at(40, zircon_math::Vec3::new(-0.8, 0.0, 0.0))],
    };
    let flat_extract = RenderHybridGiExtract {
        trace_regions: vec![trace_region_at(40, zircon_math::Vec3::new(1.6, 0.0, 0.0))],
        ..hierarchical_extract.clone()
    };

    let mut hierarchical = HybridGiRuntimeState::default();
    hierarchical.register_extract(Some(&hierarchical_extract));
    hierarchical.ingest_plan(
        57,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let mut flat = HybridGiRuntimeState::default();
    flat.register_extract(Some(&flat_extract));
    flat.ingest_plan(
        57,
        &VisibilityHybridGiUpdatePlan {
            resident_probe_ids: vec![100, 200],
            requested_probe_ids: Vec::new(),
            dirty_requested_probe_ids: Vec::new(),
            scheduled_trace_region_ids: vec![40],
            evictable_probe_ids: Vec::new(),
        },
    );

    let hierarchical_weight = hierarchical
        .build_resolve_runtime()
        .hierarchy_resolve_weight(200)
        .expect("hierarchical resolve weight");
    let flat_weight = flat
        .build_resolve_runtime()
        .hierarchy_resolve_weight(200)
        .expect("flat resolve weight");

    assert!(
        hierarchical_weight > flat_weight + 0.05,
        "expected runtime-host resolve weighting to strengthen when the current scheduled trace work still supports the probe lineage instead of leaving that scene-driven weighting entirely outside runtime resolve inputs; flat_weight={flat_weight:.3}, hierarchical_weight={hierarchical_weight:.3}"
    );
}

fn probe(probe_id: u32, resident: bool, ray_budget: u32) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity: 1,
        probe_id,
        position: zircon_math::Vec3::ZERO,
        radius: 0.5,
        parent_probe_id: None,
        resident,
        ray_budget,
    }
}

fn probe_at(
    probe_id: u32,
    resident: bool,
    ray_budget: u32,
    position: zircon_math::Vec3,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        position,
        ..probe(probe_id, resident, ray_budget)
    }
}

fn probe_with_parent(
    probe_id: u32,
    resident: bool,
    ray_budget: u32,
    parent_probe_id: u32,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        parent_probe_id: Some(parent_probe_id),
        ..probe(probe_id, resident, ray_budget)
    }
}

fn probe_with_parent_at(
    probe_id: u32,
    resident: bool,
    ray_budget: u32,
    parent_probe_id: u32,
    position: zircon_math::Vec3,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        position,
        ..probe_with_parent(probe_id, resident, ray_budget, parent_probe_id)
    }
}

fn trace_region(region_id: u32) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        entity: 1,
        region_id,
        bounds_center: zircon_math::Vec3::ZERO,
        bounds_radius: 0.5,
        screen_coverage: 1.0,
        rt_lighting_rgb: [0, 0, 0],
    }
}

fn trace_region_at(region_id: u32, bounds_center: zircon_math::Vec3) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        bounds_center,
        ..trace_region(region_id)
    }
}
