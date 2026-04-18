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
