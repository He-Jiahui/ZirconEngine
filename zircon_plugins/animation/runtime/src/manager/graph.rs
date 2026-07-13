use std::collections::HashSet;

use zircon_runtime::core::framework::animation::{AnimationGraphAsset, AnimationGraphNodeAsset};
use zircon_runtime::core::framework::animation::{
    AnimationGraphBlendMode, AnimationGraphClipInstance, AnimationGraphEvaluation,
    AnimationParameterMap,
};
use zircon_runtime::core::math::Real;

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
                let input_count = inputs.len().max(1);
                let input_weights = if input_count == 1 {
                    vec![1.0]
                } else {
                    let trailing = if input_count > 1 {
                        scalar / (input_count - 1) as Real
                    } else {
                        0.0
                    };
                    std::iter::once(1.0 - scalar)
                        .chain(std::iter::repeat_n(trailing, input_count - 1))
                        .collect::<Vec<_>>()
                };
                let mut clips = Vec::new();
                for (index, input) in inputs.iter().enumerate() {
                    let weight = input_weights.get(index).copied().unwrap_or(1.0);
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
