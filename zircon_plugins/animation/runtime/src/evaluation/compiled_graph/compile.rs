use std::sync::Arc;

use zircon_runtime::core::framework::animation::compiler::{
    compile_animation_graph, AnimationCompiledGraph, AnimationCompiledGraphNode,
};
use zircon_runtime::core::framework::animation::{AnimationGraphAsset, AnimationTargetId};
use zircon_runtime::core::framework::scene::EntityPath;

use crate::{AnimationClipCompileError, SkeletonTargetTable};

use super::error::AnimationGraphCompileError;
use super::types::{
    CompiledAnimationGraph, CompiledGraphNode, CompiledParameter, GraphNodeSlot, ParameterSlot,
};

/// Compiles source semantics once in the framework and lowers the accepted IR for evaluation.
pub fn compile_animation_graph_runtime(
    source: &AnimationGraphAsset,
    targets: Arc<SkeletonTargetTable>,
) -> Result<CompiledAnimationGraph, AnimationGraphCompileError> {
    let compilation = compile_animation_graph(source);
    let Some(artifact) = compilation.artifact() else {
        return Err(AnimationGraphCompileError::SourceDiagnostics(
            compilation.diagnostics().to_vec(),
        ));
    };
    CompiledAnimationGraph::from_compiled(artifact, targets)
}

impl CompiledAnimationGraph {
    fn from_compiled(
        artifact: &AnimationCompiledGraph,
        targets: Arc<SkeletonTargetTable>,
    ) -> Result<Self, AnimationGraphCompileError> {
        let parameters = artifact
            .parameters()
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                let slot = ParameterSlot::new(index)
                    .ok_or(AnimationGraphCompileError::ParameterCapacityExceeded)?;
                Ok(CompiledParameter {
                    name: parameter.name().to_string(),
                    default_value: parameter
                        .default_value()
                        .expect("compiled graph parameters retain authored defaults")
                        .clone(),
                })
            })
            .collect::<Result<Vec<_>, AnimationGraphCompileError>>()?;
        let nodes = artifact
            .nodes()
            .iter()
            .map(|node| compile_node(node, targets.as_ref()))
            .collect::<Result<Vec<_>, AnimationGraphCompileError>>()?;
        let output = graph_node_slot(artifact.output_node())?;
        let evaluation_order = artifact
            .evaluation_order()
            .iter()
            .copied()
            .map(graph_node_slot)
            .collect::<Result<Vec<_>, AnimationGraphCompileError>>()?;
        Ok(Self {
            parameters: parameters.into_boxed_slice(),
            nodes: nodes.into_boxed_slice(),
            output,
            evaluation_order: evaluation_order.into_boxed_slice(),
        })
    }
}

fn compile_node(
    node: &AnimationCompiledGraphNode,
    targets: &SkeletonTargetTable,
) -> Result<CompiledGraphNode, AnimationGraphCompileError> {
    match node {
        AnimationCompiledGraphNode::Clip {
            clip,
            playback_speed,
            looping,
            ..
        } => Ok(CompiledGraphNode::Clip {
            clip: clip.clone(),
            playback_speed: *playback_speed,
            looping: *looping,
        }),
        AnimationCompiledGraphNode::Blend {
            inputs,
            weight_parameter,
            ..
        } => Ok(CompiledGraphNode::Blend {
            inputs: inputs
                .iter()
                .copied()
                .map(graph_node_slot)
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
            weight_parameter: parameter_slot(*weight_parameter)?,
        }),
        AnimationCompiledGraphNode::Additive {
            base,
            additive,
            weight_parameter,
            ..
        } => Ok(CompiledGraphNode::Additive {
            base: graph_node_slot(*base)?,
            additive: graph_node_slot(*additive)?,
            weight_parameter: parameter_slot(*weight_parameter)?,
        }),
        AnimationCompiledGraphNode::Mask {
            input, target_ids, ..
        } => Ok(CompiledGraphNode::Mask {
            input: graph_node_slot(*input)?,
            target_mask: compile_mask(targets, target_ids)?,
        }),
    }
}

fn graph_node_slot(index: usize) -> Result<GraphNodeSlot, AnimationGraphCompileError> {
    GraphNodeSlot::new(index).ok_or(AnimationGraphCompileError::NodeCapacityExceeded)
}

fn parameter_slot(
    index: Option<usize>,
) -> Result<Option<ParameterSlot>, AnimationGraphCompileError> {
    index
        .map(|index| {
            ParameterSlot::new(index).ok_or(AnimationGraphCompileError::ParameterCapacityExceeded)
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
