use std::collections::HashSet;

use crate::core::framework::animation::{AnimationGraphAsset, AnimationGraphNodeAsset};
use crate::core::framework::animation::{
    AnimationGraphBlendMode, AnimationGraphClipInstance, AnimationGraphEvaluation,
    AnimationParameterMap,
};
use crate::core::math::Real;

use super::parameters::{parameter_defaults, parameter_scalar};
use super::sampling::finite_graph_clip_playback_speed;

pub(super) fn evaluate_graph(
    graph: &AnimationGraphAsset,
    overrides: &AnimationParameterMap,
) -> AnimationGraphEvaluation {
    let mut parameters = parameter_defaults(graph);
    for (name, value) in overrides {
        if super::sampling::animation_parameter_value_is_finite(value) {
            parameters.insert(name.clone(), value.clone());
        }
    }

    let output_node = graph.nodes.iter().find_map(|node| match node {
        AnimationGraphNodeAsset::Output { source } => Some(source.clone()),
        _ => None,
    });
    let clips = output_node
        .as_deref()
        .map(|source| collect_graph_clips(graph, source, &parameters, &[], &mut HashSet::new()))
        .unwrap_or_default();
    let mask_target_ids = collect_unique_graph_target_ids(&clips);

    AnimationGraphEvaluation {
        parameters,
        output_node,
        clips,
        mask_target_ids,
    }
}

fn collect_graph_clips(
    graph: &AnimationGraphAsset,
    node_id: &str,
    parameters: &AnimationParameterMap,
    inherited_target_ids: &[String],
    visited: &mut HashSet<String>,
) -> Vec<AnimationGraphClipInstance> {
    if !visited.insert(node_id.to_string()) {
        return Vec::new();
    }

    let result = graph
        .nodes
        .iter()
        .find_map(|node| match node {
            AnimationGraphNodeAsset::Clip {
                id,
                clip,
                playback_speed,
                looping,
            } if id == node_id => Some(vec![AnimationGraphClipInstance {
                clip: clip.clone(),
                playback_speed: finite_graph_clip_playback_speed(*playback_speed),
                looping: *looping,
                weight: 1.0,
                blend_mode: AnimationGraphBlendMode::Base,
                target_ids: inherited_target_ids.to_vec(),
            }]),
            AnimationGraphNodeAsset::Blend {
                id,
                inputs,
                weight_parameter,
            } if id == node_id => {
                let scalar = weight_parameter
                    .as_deref()
                    .and_then(|name| parameter_scalar(parameters, name))
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0);
                let input_count = inputs.len();
                let trailing_weight = (input_count > 1)
                    .then(|| scalar / (input_count - 1) as Real)
                    .unwrap_or(1.0);
                let mut clips = Vec::new();
                for (index, input) in inputs.iter().enumerate() {
                    let weight = if input_count <= 1 {
                        1.0
                    } else if index == 0 {
                        1.0 - scalar
                    } else {
                        trailing_weight
                    };
                    clips.extend(
                        collect_graph_clips(
                            graph,
                            input,
                            parameters,
                            inherited_target_ids,
                            visited,
                        )
                        .into_iter()
                        .map(|mut clip| {
                            clip.weight *= weight;
                            clip
                        }),
                    );
                }
                Some(clips)
            }
            AnimationGraphNodeAsset::Additive {
                id,
                base,
                additive,
                weight_parameter,
            } if id == node_id => {
                let additive_weight = weight_parameter
                    .as_deref()
                    .and_then(|name| parameter_scalar(parameters, name))
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0);
                let mut clips =
                    collect_graph_clips(graph, base, parameters, inherited_target_ids, visited);
                clips.extend(
                    collect_graph_clips(graph, additive, parameters, inherited_target_ids, visited)
                        .into_iter()
                        .map(|mut clip| {
                            clip.blend_mode = AnimationGraphBlendMode::Additive;
                            clip.weight *= additive_weight;
                            clip
                        }),
                );
                Some(clips)
            }
            AnimationGraphNodeAsset::Mask {
                id,
                input,
                target_ids,
            } if id == node_id => Some(collect_graph_clips(
                graph, input, parameters, target_ids, visited,
            )),
            _ => None,
        })
        .unwrap_or_default();

    visited.remove(node_id);
    result
}

fn collect_unique_graph_target_ids(clips: &[AnimationGraphClipInstance]) -> Vec<String> {
    let target_count = clips.iter().fold(0usize, |count, clip| {
        count.saturating_add(clip.target_ids.len())
    });
    let mut seen = HashSet::with_capacity(target_count);
    let mut target_ids = Vec::with_capacity(target_count);
    for clip in clips {
        for target_id in &clip.target_ids {
            if seen.insert(target_id.as_str()) {
                target_ids.push(target_id.clone());
            }
        }
    }
    target_ids
}

#[cfg(test)]
mod performance_contract_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::animation::{AnimationGraphBlendMode, AnimationGraphClipInstance};
    use crate::core::resource::{AssetReference, ResourceLocator};

    use super::collect_unique_graph_target_ids;

    const BENCH_SAMPLE_PAIRS: usize = 17;
    const BENCH_CLIP_COUNT: usize = 128;
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
                    .map(|target_index| {
                        format!("Rig/Hip/Spine/Bone_{clip_index:04}_{target_index:04}")
                    })
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
        let source = include_str!("graph.rs")
            .split_once("#[cfg(test)]")
            .unwrap()
            .0;

        assert!(!source.contains("let input_weights = if"));
        assert!(source.contains("let trailing_weight ="));
    }

    #[test]
    fn optimization_batch_20260826_runtime08c_graph_mask_target_dedup_uses_reserved_hash_membership()
     {
        let source = include_str!("graph.rs")
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
}
