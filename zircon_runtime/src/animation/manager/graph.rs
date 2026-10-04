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
        AnimationGraphNodeAsset::Output { source } => Some(source.as_str()),
        _ => None,
    });
    let mut visited = HashSet::with_capacity(graph.nodes.len());
    let clips = output_node
        .as_deref()
        .map(|source| collect_graph_clips(graph, source, &parameters, &[], &mut visited))
        .unwrap_or_default();
    let mask_target_ids = collect_unique_graph_target_ids(&clips);
    let output_node = output_node.map(str::to_owned);

    AnimationGraphEvaluation {
        parameters,
        output_node,
        clips,
        mask_target_ids,
    }
}

fn collect_graph_clips<'a>(
    graph: &'a AnimationGraphAsset,
    node_id: &'a str,
    parameters: &AnimationParameterMap,
    inherited_target_ids: &[String],
    visited: &mut HashSet<&'a str>,
) -> Vec<AnimationGraphClipInstance> {
    if !visited.insert(node_id) {
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
                let mut clips = Vec::with_capacity(input_count);
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
#[path = "tests/graph_performance_contract_tests.rs"]
mod performance_contract_tests;
