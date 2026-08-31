use std::collections::{HashMap, HashSet};

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

    let output_source = graph.nodes.iter().find_map(|node| match node {
        AnimationGraphNodeAsset::Output { source } => Some(source.as_str()),
        _ => None,
    });
    let node_index = build_graph_node_index(graph);
    let mut clips = Vec::with_capacity(graph.nodes.len());
    if let Some(source) = output_source {
        collect_graph_clips(
            &node_index,
            source,
            &parameters,
            &[],
            &mut HashSet::with_capacity(node_index.len()),
            &mut clips,
        );
    }
    let mask_target_ids = collect_unique_graph_target_ids(&clips);

    AnimationGraphEvaluation {
        parameters,
        output_node: output_source.map(str::to_owned),
        clips,
        mask_target_ids,
    }
}

fn build_graph_node_index(graph: &AnimationGraphAsset) -> HashMap<&str, &AnimationGraphNodeAsset> {
    let mut node_index = HashMap::with_capacity(graph.nodes.len());
    for node in &graph.nodes {
        if let Some(id) = graph_node_id(node) {
            node_index.entry(id).or_insert(node);
        }
    }
    node_index
}

fn graph_node_id(node: &AnimationGraphNodeAsset) -> Option<&str> {
    match node {
        AnimationGraphNodeAsset::Clip { id, .. }
        | AnimationGraphNodeAsset::Blend { id, .. }
        | AnimationGraphNodeAsset::Additive { id, .. }
        | AnimationGraphNodeAsset::Mask { id, .. } => Some(id.as_str()),
        AnimationGraphNodeAsset::Output { .. } => None,
    }
}

fn collect_graph_clips<'a>(
    node_index: &HashMap<&'a str, &'a AnimationGraphNodeAsset>,
    node_id: &'a str,
    parameters: &AnimationParameterMap,
    inherited_target_ids: &[String],
    visited: &mut HashSet<&'a str>,
    clips: &mut Vec<AnimationGraphClipInstance>,
) {
    if !visited.insert(node_id) {
        return;
    }

    if let Some(node) = node_index.get(node_id).copied() {
        match node {
            AnimationGraphNodeAsset::Clip {
                clip,
                playback_speed,
                looping,
                ..
            } => clips.push(AnimationGraphClipInstance {
                clip: clip.clone(),
                playback_speed: finite_graph_clip_playback_speed(*playback_speed),
                looping: *looping,
                weight: 1.0,
                blend_mode: AnimationGraphBlendMode::Base,
                target_ids: inherited_target_ids.to_vec(),
            }),
            AnimationGraphNodeAsset::Blend {
                inputs,
                weight_parameter,
                ..
            } => {
                let scalar = weight_parameter
                    .as_deref()
                    .and_then(|name| parameter_scalar(parameters, name))
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0);
                let input_count = inputs.len();
                let trailing = if input_count > 1 {
                    scalar / (input_count - 1) as Real
                } else {
                    0.0
                };
                for (index, input) in inputs.iter().enumerate() {
                    let weight = if input_count == 1 {
                        1.0
                    } else if index == 0 {
                        1.0 - scalar
                    } else {
                        trailing
                    };
                    let clip_start = clips.len();
                    collect_graph_clips(
                        node_index,
                        input,
                        parameters,
                        inherited_target_ids,
                        visited,
                        clips,
                    );
                    for clip in &mut clips[clip_start..] {
                        clip.weight *= weight;
                    }
                }
            }
            AnimationGraphNodeAsset::Additive {
                base,
                additive,
                weight_parameter,
                ..
            } => {
                let additive_weight = weight_parameter
                    .as_deref()
                    .and_then(|name| parameter_scalar(parameters, name))
                    .unwrap_or(1.0)
                    .clamp(0.0, 1.0);
                collect_graph_clips(
                    node_index,
                    base,
                    parameters,
                    inherited_target_ids,
                    visited,
                    clips,
                );
                let additive_start = clips.len();
                collect_graph_clips(
                    node_index,
                    additive,
                    parameters,
                    inherited_target_ids,
                    visited,
                    clips,
                );
                for clip in &mut clips[additive_start..] {
                    clip.blend_mode = AnimationGraphBlendMode::Additive;
                    clip.weight *= additive_weight;
                }
            }
            AnimationGraphNodeAsset::Mask {
                input, target_ids, ..
            } => collect_graph_clips(node_index, input, parameters, target_ids, visited, clips),
            AnimationGraphNodeAsset::Output { .. } => {}
        }
    }

    visited.remove(node_id);
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
