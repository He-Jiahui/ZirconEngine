use std::hint::black_box;
use std::time::Instant;

use super::{
    merge_hybrid_gi_readback_outputs, merge_particle_readback_outputs,
    merge_virtual_geometry_readback_outputs,
};
use crate::core::framework::render::{
    RenderHybridGiGlobalSdfStats, RenderHybridGiReadbackOutputs,
    RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiScenePrepareSample,
    RenderParticleGpuReadbackOutputs, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    RenderVirtualGeometryPageAssignmentRecord, RenderVirtualGeometryReadbackOutputs,
};

#[test]
fn merge_hybrid_gi_sideband_preserves_renderer_and_prepare_payloads() {
    let merged = merge_hybrid_gi_readback_outputs(
        RenderHybridGiReadbackOutputs {
            completed_probe_ids: vec![10],
            radiance_cache_gpu_stage_dispatch_counts: [1, 2, 3, 4, 5, 6],
            scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                atlas_samples: vec![RenderHybridGiScenePrepareSample {
                    index: 1,
                    rgba8: [1, 2, 3, 255],
                }],
                texture_width: 32,
                ..RenderHybridGiScenePrepareReadbackOutputs::default()
            },
            ..RenderHybridGiReadbackOutputs::default()
        },
        RenderHybridGiReadbackOutputs {
            completed_probe_ids: vec![11],
            radiance_cache_gpu_stage_dispatch_counts: [6, 5, 4, 3, 2, 1],
            scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                voxel_samples: vec![RenderHybridGiScenePrepareSample {
                    index: 4,
                    rgba8: [4, 5, 6, 255],
                }],
                texture_width: 64,
                ..RenderHybridGiScenePrepareReadbackOutputs::default()
            },
            ..RenderHybridGiReadbackOutputs::default()
        },
    );

    assert_eq!(merged.completed_probe_ids, vec![10, 11]);
    assert_eq!(merged.radiance_cache_gpu_stage_dispatch_counts, [7; 6]);
    assert_eq!(merged.scene_prepare.atlas_samples.len(), 1);
    assert_eq!(merged.scene_prepare.voxel_samples.len(), 1);
    assert_eq!(merged.scene_prepare.texture_width, 64);
}

#[test]
fn merge_hybrid_gi_sideband_preserves_renderer_global_sdf_stats() {
    let merged = merge_hybrid_gi_readback_outputs(
        RenderHybridGiReadbackOutputs {
            global_sdf_stats: Some(RenderHybridGiGlobalSdfStats {
                resident_page_count: 8,
                uploaded_page_count: 2,
                ..RenderHybridGiGlobalSdfStats::default()
            }),
            ..RenderHybridGiReadbackOutputs::default()
        },
        RenderHybridGiReadbackOutputs {
            global_sdf_stats: Some(RenderHybridGiGlobalSdfStats {
                resident_page_count: 1,
                ..RenderHybridGiGlobalSdfStats::default()
            }),
            ..RenderHybridGiReadbackOutputs::default()
        },
    );

    let stats = merged
        .global_sdf_stats
        .expect("renderer Global SDF stats must remain authoritative");
    assert_eq!(stats.resident_page_count, 8);
    assert_eq!(stats.uploaded_page_count, 2);
}

#[test]
fn merge_virtual_geometry_sideband_preserves_node_cluster_page_requests() {
    let merged = merge_virtual_geometry_readback_outputs(
        RenderVirtualGeometryReadbackOutputs {
            completed_page_assignments: vec![RenderVirtualGeometryPageAssignmentRecord {
                page_id: 42,
                physical_slot: 3,
            }],
            ..RenderVirtualGeometryReadbackOutputs::default()
        },
        RenderVirtualGeometryReadbackOutputs {
            node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                page_request_ids: vec![300, 301],
                ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
            },
            ..RenderVirtualGeometryReadbackOutputs::default()
        },
    );

    assert_eq!(merged.completed_page_assignments.len(), 1);
    assert_eq!(merged.node_cluster_cull.page_request_ids, vec![300, 301]);
}

#[test]
fn merge_particle_sideband_uses_renderer_payload_as_authority() {
    let sideband = RenderParticleGpuReadbackOutputs {
        alive_count: 2,
        spawned_total: 2,
        per_emitter_spawned: vec![2],
        indirect_draw_args: [6, 2, 0, 0],
        ..RenderParticleGpuReadbackOutputs::default()
    };
    let renderer = RenderParticleGpuReadbackOutputs {
        alive_count: 4,
        spawned_total: 4,
        per_emitter_spawned: vec![4],
        indirect_draw_args: [6, 4, 0, 0],
        ..RenderParticleGpuReadbackOutputs::default()
    };

    assert_eq!(
        merge_particle_readback_outputs(
            RenderParticleGpuReadbackOutputs::default(),
            sideband.clone()
        ),
        sideband
    );
    assert_eq!(
        merge_particle_readback_outputs(renderer.clone(), sideband),
        renderer
    );
}

#[test]
fn optimization_batch_dn_hybrid_readback_append_preserves_owned_vectors() {
    let source = include_str!("../collect_runtime_feedback.rs");
    let merge = source
        .split("fn merge_hybrid_gi_readback_outputs")
        .nth(1)
        .expect("hybrid readback merge")
        .split("fn merge_particle_readback_outputs")
        .next()
        .expect("hybrid readback merge body");
    let prepare = source
        .split("fn append_hybrid_gi_scene_prepare_readback")
        .nth(1)
        .expect("scene prepare merge")
        .split("fn merge_virtual_geometry_readback_outputs")
        .next()
        .expect("scene prepare merge body");

    assert!(merge.matches(".append(&mut").count() >= 5);
    assert!(prepare.matches(".append(&mut").count() >= 15);
    assert!(!merge.contains(".extend(") && !prepare.contains(".extend("));
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_dn_hybrid_readback_append_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const MERGES_PER_SAMPLE: usize = 8_192;
    const VALUES_PER_VECTOR: usize = 4_096;

    let template = (0..VALUES_PER_VECTOR as u64).collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_hybrid_readback_merge(
                &template,
                MERGES_PER_SAMPLE,
                true,
            ));
            optimized_samples.push(measure_hybrid_readback_merge(
                &template,
                MERGES_PER_SAMPLE,
                false,
            ));
        } else {
            optimized_samples.push(measure_hybrid_readback_merge(
                &template,
                MERGES_PER_SAMPLE,
                false,
            ));
            legacy_samples.push(measure_hybrid_readback_merge(
                &template,
                MERGES_PER_SAMPLE,
                true,
            ));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "RUNTIME422_HYBRID_READBACK_APPEND_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "hybrid readback append p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

fn measure_hybrid_readback_merge(template: &[u64], merge_count: usize, legacy: bool) -> u128 {
    let started_at = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..merge_count {
        let mut target = vec![black_box(1_u64)];
        let mut incoming = black_box(template).to_vec();
        if legacy {
            target.extend(incoming);
        } else {
            target.append(&mut incoming);
        }
        checksum = checksum.wrapping_add(black_box(target.len()) as u64);
    }
    black_box(checksum);
    started_at.elapsed().as_nanos()
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}
