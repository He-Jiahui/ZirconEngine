use std::hint::black_box;
use std::time::Instant;

use super::merge_plugin_renderer_outputs;
use crate::core::framework::render::{
    RenderHybridGiCacheEntryRecord, RenderHybridGiGlobalSdfStats, RenderHybridGiReadbackOutputs,
    RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiScenePrepareSample,
    RenderParticleGpuReadbackOutputs, RenderPluginRendererOutputs,
    RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryPageAssignmentRecord,
    RenderVirtualGeometryReadbackOutputs,
};

#[test]
fn merge_preserves_hybrid_gi_runtime_prepare_and_graph_payloads() {
    let mut base = RenderPluginRendererOutputs {
        hybrid_gi: RenderHybridGiReadbackOutputs {
            cache_entries: vec![RenderHybridGiCacheEntryRecord { key: 5, value: 7 }],
            radiance_cache_gpu_stage_dispatch_counts: [1, 2, 3, 4, 5, 6],
            scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                surface_cache_depth_samples: vec![RenderHybridGiScenePrepareSample {
                    index: 1,
                    rgba8: [96, 96, 96, 255],
                }],
                probe_trace_dispatch: [1, 1, 1],
                texture_width: 64,
                ..RenderHybridGiScenePrepareReadbackOutputs::default()
            },
            ..RenderHybridGiReadbackOutputs::default()
        },
        ..RenderPluginRendererOutputs::default()
    };

    merge_plugin_renderer_outputs(
        &mut base,
        RenderPluginRendererOutputs {
            hybrid_gi: RenderHybridGiReadbackOutputs {
                completed_probe_ids: vec![11],
                radiance_cache_gpu_stage_dispatch_counts: [6, 5, 4, 3, 2, 1],
                scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                    atlas_samples: vec![RenderHybridGiScenePrepareSample {
                        index: 2,
                        rgba8: [10, 20, 30, 255],
                    }],
                    probe_trace_dispatch: [1, 1, 3],
                    texture_width: 128,
                    ..RenderHybridGiScenePrepareReadbackOutputs::default()
                },
                ..RenderHybridGiReadbackOutputs::default()
            },
            ..RenderPluginRendererOutputs::default()
        },
    );

    assert_eq!(
        base.hybrid_gi.cache_entries,
        vec![RenderHybridGiCacheEntryRecord { key: 5, value: 7 }]
    );
    assert_eq!(base.hybrid_gi.completed_probe_ids, vec![11]);
    assert_eq!(
        base.hybrid_gi.radiance_cache_gpu_stage_dispatch_counts,
        [7; 6]
    );
    assert_eq!(base.hybrid_gi.scene_prepare.atlas_samples.len(), 1);
    assert_eq!(
        base.hybrid_gi
            .scene_prepare
            .surface_cache_depth_samples
            .len(),
        1
    );
    assert_eq!(base.hybrid_gi.scene_prepare.probe_trace_dispatch, [1, 1, 3]);
    assert_eq!(base.hybrid_gi.scene_prepare.texture_width, 128);
}

#[test]
fn merge_replaces_global_sdf_stats_with_the_latest_renderer_output() {
    let mut base = RenderPluginRendererOutputs {
        hybrid_gi: RenderHybridGiReadbackOutputs {
            global_sdf_stats: Some(RenderHybridGiGlobalSdfStats {
                resident_page_count: 2,
                ..RenderHybridGiGlobalSdfStats::default()
            }),
            ..RenderHybridGiReadbackOutputs::default()
        },
        ..RenderPluginRendererOutputs::default()
    };

    merge_plugin_renderer_outputs(
        &mut base,
        RenderPluginRendererOutputs {
            hybrid_gi: RenderHybridGiReadbackOutputs {
                global_sdf_stats: Some(RenderHybridGiGlobalSdfStats {
                    resident_page_count: 5,
                    dirty_page_count: 1,
                    ..RenderHybridGiGlobalSdfStats::default()
                }),
                ..RenderHybridGiReadbackOutputs::default()
            },
            ..RenderPluginRendererOutputs::default()
        },
    );

    let stats = base.hybrid_gi.global_sdf_stats.unwrap();
    assert_eq!(stats.resident_page_count, 5);
    assert_eq!(stats.dirty_page_count, 1);
}

#[test]
fn merge_appends_virtual_geometry_readback_payloads() {
    let mut base = RenderPluginRendererOutputs {
        virtual_geometry: RenderVirtualGeometryReadbackOutputs {
            completed_page_assignments: vec![RenderVirtualGeometryPageAssignmentRecord {
                page_id: 4,
                physical_slot: 1,
            }],
            ..RenderVirtualGeometryReadbackOutputs::default()
        },
        ..RenderPluginRendererOutputs::default()
    };

    merge_plugin_renderer_outputs(
        &mut base,
        RenderPluginRendererOutputs {
            virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                page_table_entries: vec![9],
                node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                    page_request_ids: vec![300],
                    ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                },
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
            ..RenderPluginRendererOutputs::default()
        },
    );

    assert_eq!(base.virtual_geometry.completed_page_assignments.len(), 1);
    assert_eq!(base.virtual_geometry.page_table_entries, vec![9]);
    assert_eq!(
        base.virtual_geometry.node_cluster_cull.page_request_ids,
        vec![300]
    );
}

#[test]
fn merge_keeps_particle_outputs_single_owner() {
    let mut base = RenderPluginRendererOutputs {
        particles: RenderParticleGpuReadbackOutputs {
            alive_count: 2,
            spawned_total: 2,
            ..RenderParticleGpuReadbackOutputs::default()
        },
        ..RenderPluginRendererOutputs::default()
    };

    merge_plugin_renderer_outputs(
        &mut base,
        RenderPluginRendererOutputs {
            particles: RenderParticleGpuReadbackOutputs {
                alive_count: 5,
                spawned_total: 5,
                ..RenderParticleGpuReadbackOutputs::default()
            },
            ..RenderPluginRendererOutputs::default()
        },
    );

    assert_eq!(base.particles.alive_count, 5);
    assert_eq!(base.particles.spawned_total, 5);
}

#[test]
fn optimization_batch_dp_plugin_readback_merge_uses_owned_append_paths() {
    let source = include_str!("../merge_plugin_renderer_outputs.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("plugin readback merge production source");
    assert!(!production.contains(".extend("));
    assert!(production.matches(".append(&mut").count() >= 25);
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_dp_plugin_readback_owned_append_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const MERGES_PER_SAMPLE: usize = 4_096;
    const VALUES_PER_MERGE: usize = 1_024;

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_owned_merge(
                MERGES_PER_SAMPLE,
                VALUES_PER_MERGE,
                false,
            ));
            optimized_samples.push(measure_owned_merge(
                MERGES_PER_SAMPLE,
                VALUES_PER_MERGE,
                true,
            ));
        } else {
            optimized_samples.push(measure_owned_merge(
                MERGES_PER_SAMPLE,
                VALUES_PER_MERGE,
                true,
            ));
            legacy_samples.push(measure_owned_merge(
                MERGES_PER_SAMPLE,
                VALUES_PER_MERGE,
                false,
            ));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "RUNTIME424_PLUGIN_READBACK_OWNED_APPEND_BENCH_V1 merges_per_sample={MERGES_PER_SAMPLE} values_per_merge={VALUES_PER_MERGE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "plugin readback owned append p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

fn measure_owned_merge(merge_count: usize, values_per_merge: usize, append: bool) -> u128 {
    let started_at = Instant::now();
    let mut checksum = 0_usize;
    for merge_index in 0..merge_count {
        let mut base = Vec::new();
        let mut incoming = (0..values_per_merge)
            .map(|value| value ^ merge_index)
            .collect::<Vec<_>>();
        if append {
            base.append(&mut incoming);
        } else {
            base.extend(incoming);
        }
        checksum = checksum.wrapping_add(base.len() ^ base.capacity());
        black_box(&base);
    }
    black_box(checksum);
    started_at.elapsed().as_nanos()
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}
