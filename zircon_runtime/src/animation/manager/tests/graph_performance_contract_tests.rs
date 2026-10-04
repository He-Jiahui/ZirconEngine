use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::animation::{
    AnimationGraphAsset, AnimationGraphBlendMode, AnimationGraphClipInstance,
    AnimationGraphNodeAsset, AnimationParameterMap,
};
use crate::core::resource::{AssetReference, ResourceLocator};

use super::{collect_unique_graph_target_ids, evaluate_graph};

const BENCH_SAMPLE_PAIRS: usize = 17;
const BENCH_CLIP_COUNT: usize = 128;
const BENCH_GRAPH_NODE_COUNT: usize = 32_768;
const BENCH_GRAPH_TRAVERSAL_ITERATIONS: usize = 32;
const BENCH_TARGETS_PER_CLIP: usize = 16;

fn clip(target_ids: &[&str]) -> AnimationGraphClipInstance {
    AnimationGraphClipInstance {
        clip: AssetReference::from_locator(
            ResourceLocator::parse("res://animation/graph-target-bench.clip").unwrap(),
        ),
        playback_speed: 1.0,
        looping: true,
        weight: 1.0,
        blend_mode: AnimationGraphBlendMode::Base,
        target_ids: target_ids
            .iter()
            .map(|target_id| (*target_id).to_string())
            .collect(),
    }
}

fn legacy_collect_unique_graph_target_ids(clips: &[AnimationGraphClipInstance]) -> Vec<String> {
    let mut target_ids = Vec::new();
    for clip in clips {
        for target_id in &clip.target_ids {
            if !target_ids.iter().any(|existing| existing == target_id) {
                target_ids.push(target_id.clone());
            }
        }
    }
    target_ids
}

fn benchmark_clips() -> Vec<AnimationGraphClipInstance> {
    let clip_reference = AssetReference::from_locator(
        ResourceLocator::parse("res://animation/graph-target-bench.clip").unwrap(),
    );
    (0..BENCH_CLIP_COUNT)
        .map(|clip_index| AnimationGraphClipInstance {
            clip: clip_reference.clone(),
            playback_speed: 1.0,
            looping: true,
            weight: 1.0,
            blend_mode: AnimationGraphBlendMode::Base,
            target_ids: (0..BENCH_TARGETS_PER_CLIP)
                .map(|target_index| format!("Rig/Hip/Spine/Bone_{clip_index:04}_{target_index:04}"))
                .collect(),
        })
        .collect()
}

fn elapsed_micros(run: impl FnOnce()) -> u128 {
    let started = Instant::now();
    run();
    started.elapsed().as_micros()
}

fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * 95).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
fn blend_weights_do_not_allocate_a_temporary_vector() {
    let source = include_str!("../graph.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(!source.contains("let input_weights = if"));
    assert!(source.contains("let trailing_weight ="));
}

#[test]
fn blend_clip_aggregation_reserves_input_lower_bound() {
    let source = include_str!("../graph.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("let mut clips = Vec::with_capacity(input_count);"));
    assert!(!source.contains("let mut clips = Vec::new();"));
}

#[test]
fn optimization_batch_20260826_runtime08c_graph_mask_target_dedup_uses_reserved_hash_membership() {
    let source = include_str!("../graph.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("HashSet::with_capacity(target_count)"));
    assert!(source.contains("Vec::with_capacity(target_count)"));
    assert!(source.contains("seen.insert(target_id.as_str())"));
    assert!(!source.contains("target_ids.iter().any(|existing| existing == target_id)"));
}

#[test]
fn optimization_batch_20260826_runtime08c_graph_mask_target_dedup_preserves_first_seen_order() {
    let clips = vec![
        clip(&["Rig/Hip", "Rig/Arm", "Rig/Hip"]),
        clip(&["Rig/Arm", "Rig/Leg", "Rig/Head", "Rig/Leg"]),
    ];

    assert_eq!(
        collect_unique_graph_target_ids(&clips),
        legacy_collect_unique_graph_target_ids(&clips)
    );
    assert_eq!(
        collect_unique_graph_target_ids(&clips),
        ["Rig/Hip", "Rig/Arm", "Rig/Leg", "Rig/Head"]
    );
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826_runtime08c_graph_mask_target_dedup_performance_evidence() {
    let clips = benchmark_clips();
    let expected_target_count = BENCH_CLIP_COUNT * BENCH_TARGETS_PER_CLIP;
    assert_eq!(
        collect_unique_graph_target_ids(&clips).len(),
        expected_target_count
    );

    for _ in 0..4 {
        black_box(legacy_collect_unique_graph_target_ids(black_box(&clips)));
        black_box(collect_unique_graph_target_ids(black_box(&clips)));
    }

    let mut legacy_samples = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
    for sample_index in 0..BENCH_SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(elapsed_micros(|| {
                black_box(legacy_collect_unique_graph_target_ids(black_box(&clips)));
            }));
            optimized_samples.push(elapsed_micros(|| {
                black_box(collect_unique_graph_target_ids(black_box(&clips)));
            }));
        } else {
            optimized_samples.push(elapsed_micros(|| {
                black_box(collect_unique_graph_target_ids(black_box(&clips)));
            }));
            legacy_samples.push(elapsed_micros(|| {
                black_box(legacy_collect_unique_graph_target_ids(black_box(&clips)));
            }));
        }
    }

    let legacy_p95 = nearest_rank_p95(&mut legacy_samples);
    let optimized_p95 = nearest_rank_p95(&mut optimized_samples);
    println!(
        "RUNTIME08C_GRAPH_MASK_TARGET_DEDUP_BENCH_V1 sample_pairs={} clips={} targets_per_clip={} total_targets={} legacy_p95_us={} optimized_p95_us={} legacy_samples_us={:?} optimized_samples_us={:?}",
        BENCH_SAMPLE_PAIRS,
        BENCH_CLIP_COUNT,
        BENCH_TARGETS_PER_CLIP,
        expected_target_count,
        legacy_p95,
        optimized_p95,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(35),
        "hash-based target dedup p95 must be at least 65% below the quadratic legacy path: legacy={legacy_p95}us optimized={optimized_p95}us"
    );
}

#[test]
fn optimization_batch_in_runtime624_graph_traversal_preallocates_visited_membership() {
    let source = include_str!("../graph.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("HashSet::with_capacity(graph.nodes.len())"));
    assert!(!source.contains("&mut HashSet::new()"));
}

#[test]
fn graph_traversal_uses_borrowed_membership_keys() {
    let source = include_str!("../graph.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("visited: &mut HashSet<&'a str>"));
    assert!(source.contains("visited.insert(node_id)"));
    assert!(!source.contains("visited.insert(node_id.to_string())"));
}

#[test]
fn borrowed_graph_traversal_preserves_owned_output_and_clip_projection() {
    let graph = AnimationGraphAsset {
        name: Some("borrowed-membership-test".to_string()),
        parameters: Vec::new(),
        nodes: vec![
            AnimationGraphNodeAsset::Output {
                source: "blend".to_string(),
            },
            AnimationGraphNodeAsset::Blend {
                id: "blend".to_string(),
                inputs: vec!["clip".to_string()],
                weight_parameter: None,
            },
            AnimationGraphNodeAsset::Clip {
                id: "clip".to_string(),
                clip: crate::core::resource::AssetReference::from_locator(
                    ResourceLocator::parse("res://animation/borrowed-membership-test.clip")
                        .unwrap(),
                ),
                playback_speed: 1.0,
                looping: true,
            },
        ],
    };

    let evaluation = evaluate_graph(&graph, &AnimationParameterMap::default());

    assert_eq!(evaluation.output_node.as_deref(), Some("blend"));
    assert_eq!(evaluation.clips.len(), 1);
    assert_eq!(evaluation.clips[0].weight, 1.0);
    assert!(evaluation.mask_target_ids.is_empty());
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_in_runtime624_preallocated_graph_traversal_performance_evidence() {
    let node_ids = (0..BENCH_GRAPH_NODE_COUNT)
        .map(|index| format!("animation.graph.node.{index:05}"))
        .collect::<Vec<_>>();
    let mut unreserved_samples = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
    for sample_index in 0..BENCH_SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            unreserved_samples.push(measure_graph_visited_membership(&node_ids, false));
            preallocated_samples.push(measure_graph_visited_membership(&node_ids, true));
        } else {
            preallocated_samples.push(measure_graph_visited_membership(&node_ids, true));
            unreserved_samples.push(measure_graph_visited_membership(&node_ids, false));
        }
    }

    let unreserved_p95 = nearest_rank_p95(&mut unreserved_samples);
    let preallocated_p95 = nearest_rank_p95(&mut preallocated_samples);
    println!(
        "RUNTIME624_PREALLOCATED_ANIMATION_GRAPH_TRAVERSAL_BENCH_V1 sample_pairs={} nodes={} iterations={} unreserved_p95_us={} preallocated_p95_us={} unreserved_samples_us={:?} preallocated_samples_us={:?}",
        BENCH_SAMPLE_PAIRS,
        BENCH_GRAPH_NODE_COUNT,
        BENCH_GRAPH_TRAVERSAL_ITERATIONS,
        unreserved_p95,
        preallocated_p95,
        unreserved_samples,
        preallocated_samples,
    );
    assert!(
        preallocated_p95.saturating_mul(100) <= unreserved_p95.saturating_mul(85),
        "preallocated graph traversal membership p95 must be at most 85% of unreserved: unreserved={unreserved_p95}us preallocated={preallocated_p95}us"
    );
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_in_runtime945_borrowed_graph_membership_performance_evidence() {
    let node_ids = (0..BENCH_GRAPH_NODE_COUNT)
        .map(|index| format!("animation.graph.node.{index:05}"))
        .collect::<Vec<_>>();
    let mut owned_samples = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
    let mut borrowed_samples = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
    for sample_index in 0..BENCH_SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            owned_samples.push(measure_graph_membership_key_ownership(&node_ids, false));
            borrowed_samples.push(measure_graph_membership_key_ownership(&node_ids, true));
        } else {
            borrowed_samples.push(measure_graph_membership_key_ownership(&node_ids, true));
            owned_samples.push(measure_graph_membership_key_ownership(&node_ids, false));
        }
    }

    let owned_p95 = nearest_rank_p95(&mut owned_samples);
    let borrowed_p95 = nearest_rank_p95(&mut borrowed_samples);
    println!(
        "RUNTIME945_GRAPH_BORROWED_MEMBERSHIP_BENCH_V1 sample_pairs={} nodes={} iterations={} owned_key_allocations_per_sample={} borrowed_key_allocations_per_sample=0 owned_p95_us={} borrowed_p95_us={} owned_samples_us={:?} borrowed_samples_us={:?}",
        BENCH_SAMPLE_PAIRS,
        BENCH_GRAPH_NODE_COUNT,
        BENCH_GRAPH_TRAVERSAL_ITERATIONS,
        BENCH_GRAPH_NODE_COUNT * BENCH_GRAPH_TRAVERSAL_ITERATIONS,
        owned_p95,
        borrowed_p95,
        owned_samples,
        borrowed_samples,
    );
    assert!(
        borrowed_p95.saturating_mul(100) <= owned_p95.saturating_mul(85),
        "borrowed graph membership p95 must be at most 85% of owned-key baseline: owned={owned_p95}us borrowed={borrowed_p95}us"
    );
}

fn measure_graph_membership_key_ownership(node_ids: &[String], borrowed: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..BENCH_GRAPH_TRAVERSAL_ITERATIONS {
        if borrowed {
            let mut visited = HashSet::with_capacity(node_ids.len());
            for node_id in node_ids {
                black_box(visited.insert(black_box(node_id.as_str())));
            }
            black_box(visited.len());
        } else {
            let mut visited = HashSet::with_capacity(node_ids.len());
            for node_id in node_ids {
                black_box(visited.insert(black_box(node_id.to_string())));
            }
            black_box(visited.len());
        }
    }
    started.elapsed().as_micros().max(1)
}

fn measure_graph_visited_membership(node_ids: &[String], preallocated: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..BENCH_GRAPH_TRAVERSAL_ITERATIONS {
        let mut visited = if preallocated {
            HashSet::with_capacity(node_ids.len())
        } else {
            HashSet::new()
        };
        for node_id in node_ids {
            black_box(visited.insert(black_box(node_id.as_str())));
        }
        black_box(visited.len());
    }
    started.elapsed().as_micros().max(1)
}
