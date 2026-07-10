use std::collections::BTreeMap;
use std::sync::Arc;

use zircon_runtime::asset::{AnimationGraphAsset, AnimationGraphNodeAsset};
use zircon_runtime::core::framework::animation::AnimationTargetId;
use zircon_runtime::core::framework::scene::EntityPath;

use crate::{AnimationClipCompileError, SkeletonTargetTable};

use super::error::AnimationGraphCompileError;
use super::types::{
    CompiledAnimationGraph, CompiledGraphNode, CompiledParameter, GraphNodeSlot, ParameterSlot,
};

impl CompiledAnimationGraph {
    pub fn compile(
        graph: &AnimationGraphAsset,
        targets: Arc<SkeletonTargetTable>,
    ) -> Result<Self, AnimationGraphCompileError> {
        let (parameters, parameter_slots) = compile_parameters(graph)?;
        let (node_slots, node_names) = bind_nodes(graph)?;
        let mut output = None;
        let mut nodes = Vec::with_capacity(node_names.len());

        for source in &graph.nodes {
            match source {
                AnimationGraphNodeAsset::Output { source } => {
                    if output.is_some() {
                        return Err(AnimationGraphCompileError::DuplicateOutput);
                    }
                    output = Some(resolve_node(&node_slots, source)?);
                }
                _ => nodes.push(compile_node(
                    source,
                    &node_slots,
                    &parameter_slots,
                    targets.as_ref(),
                )?),
            }
        }

        let output = output.ok_or(AnimationGraphCompileError::MissingOutput)?;
        validate_acyclic(&nodes, &node_names, output)?;
        Ok(Self {
            parameters: parameters.into_boxed_slice(),
            nodes: nodes.into_boxed_slice(),
            output,
        })
    }
}

fn compile_parameters(
    graph: &AnimationGraphAsset,
) -> Result<(Vec<CompiledParameter>, BTreeMap<String, ParameterSlot>), AnimationGraphCompileError> {
    let mut compiled = Vec::with_capacity(graph.parameters.len());
    let mut slots = BTreeMap::new();
    for parameter in &graph.parameters {
        let slot = ParameterSlot::new(compiled.len())
            .ok_or(AnimationGraphCompileError::ParameterCapacityExceeded)?;
        if slots.insert(parameter.name.clone(), slot).is_some() {
            return Err(AnimationGraphCompileError::DuplicateParameter {
                name: parameter.name.clone(),
            });
        }
        compiled.push(CompiledParameter {
            name: parameter.name.clone(),
            default_value: parameter.default_value.clone(),
        });
    }
    Ok((compiled, slots))
}

fn bind_nodes(
    graph: &AnimationGraphAsset,
) -> Result<(BTreeMap<String, GraphNodeSlot>, Vec<String>), AnimationGraphCompileError> {
    let mut slots = BTreeMap::new();
    let mut names = Vec::new();
    for node in &graph.nodes {
        let Some(name) = node_name(node) else {
            continue;
        };
        let slot = GraphNodeSlot::new(names.len())
            .ok_or(AnimationGraphCompileError::NodeCapacityExceeded)?;
        if slots.insert(name.to_string(), slot).is_some() {
            return Err(AnimationGraphCompileError::DuplicateNode { name: name.into() });
        }
        names.push(name.to_string());
    }
    Ok((slots, names))
}

fn node_name(node: &AnimationGraphNodeAsset) -> Option<&str> {
    match node {
        AnimationGraphNodeAsset::Clip { id, .. }
        | AnimationGraphNodeAsset::Blend { id, .. }
        | AnimationGraphNodeAsset::Additive { id, .. }
        | AnimationGraphNodeAsset::Mask { id, .. } => Some(id),
        AnimationGraphNodeAsset::Output { .. } => None,
    }
}

fn compile_node(
    node: &AnimationGraphNodeAsset,
    nodes: &BTreeMap<String, GraphNodeSlot>,
    parameters: &BTreeMap<String, ParameterSlot>,
    targets: &SkeletonTargetTable,
) -> Result<CompiledGraphNode, AnimationGraphCompileError> {
    match node {
        AnimationGraphNodeAsset::Clip {
            clip,
            playback_speed,
            looping,
            ..
        } => Ok(CompiledGraphNode::Clip {
            clip: clip.clone(),
            playback_speed: finite_playback_speed(*playback_speed),
            looping: *looping,
        }),
        AnimationGraphNodeAsset::Blend {
            inputs,
            weight_parameter,
            ..
        } => Ok(CompiledGraphNode::Blend {
            inputs: inputs
                .iter()
                .map(|name| resolve_node(nodes, name))
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
            weight_parameter: resolve_parameter(parameters, weight_parameter.as_deref())?,
        }),
        AnimationGraphNodeAsset::Additive {
            base,
            additive,
            weight_parameter,
            ..
        } => Ok(CompiledGraphNode::Additive {
            base: resolve_node(nodes, base)?,
            additive: resolve_node(nodes, additive)?,
            weight_parameter: resolve_parameter(parameters, weight_parameter.as_deref())?,
        }),
        AnimationGraphNodeAsset::Mask {
            input, target_ids, ..
        } => Ok(CompiledGraphNode::Mask {
            input: resolve_node(nodes, input)?,
            target_mask: compile_mask(targets, target_ids)?,
        }),
        AnimationGraphNodeAsset::Output { .. } => {
            Err(AnimationGraphCompileError::UnexpectedOutputNode)
        }
    }
}

fn resolve_node(
    nodes: &BTreeMap<String, GraphNodeSlot>,
    name: &str,
) -> Result<GraphNodeSlot, AnimationGraphCompileError> {
    nodes
        .get(name)
        .copied()
        .ok_or_else(|| AnimationGraphCompileError::MissingNode { name: name.into() })
}

fn resolve_parameter(
    parameters: &BTreeMap<String, ParameterSlot>,
    name: Option<&str>,
) -> Result<Option<ParameterSlot>, AnimationGraphCompileError> {
    name.map(|name| {
        parameters
            .get(name)
            .copied()
            .ok_or_else(|| AnimationGraphCompileError::MissingParameter { name: name.into() })
    })
    .transpose()
}

fn compile_mask(
    targets: &SkeletonTargetTable,
    names: &[String],
) -> Result<Arc<[bool]>, AnimationGraphCompileError> {
    let mut mask = vec![names.is_empty(); targets.len()];
    for name in names {
        let bone_index = if name.contains('/') {
            let path = EntityPath::parse(name).map_err(|_| {
                AnimationGraphCompileError::InvalidMaskTarget {
                    target: name.clone(),
                }
            })?;
            targets
                .bone_index_for_target(AnimationTargetId::from_path(&path))
                .ok_or_else(|| AnimationGraphCompileError::UnresolvedMaskTarget {
                    target: name.clone(),
                })?
        } else {
            let slot = targets
                .resolve_unique_bone_name(0, name)
                .map_err(|error| match error {
                    AnimationClipCompileError::AmbiguousTrack { .. } => {
                        AnimationGraphCompileError::AmbiguousMaskTarget {
                            target: name.clone(),
                        }
                    }
                    _ => AnimationGraphCompileError::UnresolvedMaskTarget {
                        target: name.clone(),
                    },
                })?;
            targets.bone_index_for_slot(slot).ok_or_else(|| {
                AnimationGraphCompileError::UnresolvedMaskTarget {
                    target: name.clone(),
                }
            })?
        };
        mask[bone_index] = true;
    }
    Ok(mask.into())
}

fn finite_playback_speed(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        1.0
    }
}

fn validate_acyclic(
    nodes: &[CompiledGraphNode],
    names: &[String],
    output: GraphNodeSlot,
) -> Result<(), AnimationGraphCompileError> {
    fn visit(
        slot: GraphNodeSlot,
        nodes: &[CompiledGraphNode],
        names: &[String],
        visiting: &mut [bool],
        visited: &mut [bool],
    ) -> Result<(), AnimationGraphCompileError> {
        let index = slot.index();
        if visiting[index] {
            return Err(AnimationGraphCompileError::Cycle {
                name: names[index].clone(),
            });
        }
        if visited[index] {
            return Ok(());
        }
        visiting[index] = true;
        match &nodes[index] {
            CompiledGraphNode::Clip { .. } => {}
            CompiledGraphNode::Blend { inputs, .. } => {
                for input in inputs {
                    visit(*input, nodes, names, visiting, visited)?;
                }
            }
            CompiledGraphNode::Additive { base, additive, .. } => {
                visit(*base, nodes, names, visiting, visited)?;
                visit(*additive, nodes, names, visiting, visited)?;
            }
            CompiledGraphNode::Mask { input, .. } => {
                visit(*input, nodes, names, visiting, visited)?
            }
        }
        visiting[index] = false;
        visited[index] = true;
        Ok(())
    }
    let mut visiting = vec![false; nodes.len()];
    let mut visited = vec![false; nodes.len()];
    visit(output, nodes, names, &mut visiting, &mut visited)
}
