use std::collections::BTreeMap;
use std::sync::Arc;

use zircon_runtime::core::framework::animation::{
    AnimationGraphBlendMode, AnimationParameterMap, AnimationParameterValue,
};
use zircon_runtime::core::math::Real;

use super::types::{
    CompiledAnimationGraph, CompiledAnimationGraphEvaluation, CompiledGraphClipInstance,
    CompiledGraphNode, CompiledParameter, GraphNodeSlot, ParameterSlot,
};

impl CompiledAnimationGraph {
    pub fn evaluate(&self, overrides: &AnimationParameterMap) -> CompiledAnimationGraphEvaluation {
        let mut weights_by_node = (0..self.nodes.len())
            .map(|_| GraphContextWeights::default())
            .collect::<Vec<_>>();
        weights_by_node[self.output.index()].accumulate(GraphEvaluationContext::default(), 1.0);
        let mut contributions = Vec::new();

        for slot in self.evaluation_order.iter().rev().copied() {
            let incoming = std::mem::take(&mut weights_by_node[slot.index()]);
            match &self.nodes[slot.index()] {
                CompiledGraphNode::Clip { .. } => {
                    incoming.for_each(|context, weight| {
                        contributions.push(GraphClipContribution {
                            clip: slot,
                            context,
                            weight,
                        });
                    });
                }
                CompiledGraphNode::Blend {
                    inputs,
                    weight_parameter,
                } => {
                    let scalar = parameter_scalar(&self.parameters, overrides, *weight_parameter)
                        .unwrap_or(1.0)
                        .clamp(0.0, 1.0);
                    let input_count = inputs.len().max(1);
                    for (index, input) in inputs.iter().copied().enumerate() {
                        let scale = if input_count == 1 {
                            1.0
                        } else if index == 0 {
                            1.0 - scalar
                        } else {
                            scalar / (input_count - 1) as Real
                        };
                        incoming.propagate(&mut weights_by_node[input.index()], scale, None, false);
                    }
                }
                CompiledGraphNode::Additive {
                    base,
                    additive,
                    weight_parameter,
                } => {
                    incoming.propagate(&mut weights_by_node[base.index()], 1.0, None, false);
                    let scale = parameter_scalar(&self.parameters, overrides, *weight_parameter)
                        .unwrap_or(1.0)
                        .clamp(0.0, 1.0);
                    incoming.propagate(&mut weights_by_node[additive.index()], scale, None, true);
                }
                CompiledGraphNode::Mask { input, .. } => {
                    incoming.propagate(&mut weights_by_node[input.index()], 1.0, Some(slot), false)
                }
            }
        }

        contributions.sort_unstable_by_key(|contribution| {
            (
                contribution.context.additive,
                contribution.clip,
                contribution.context.target_mask,
            )
        });
        let clips = contributions
            .into_iter()
            .map(|contribution| self.materialize_clip_contribution(contribution))
            .collect();
        CompiledAnimationGraphEvaluation { clips }
    }

    fn materialize_clip_contribution(
        &self,
        contribution: GraphClipContribution,
    ) -> CompiledGraphClipInstance {
        let GraphClipContribution {
            clip: clip_slot,
            context,
            weight,
        } = contribution;
        let CompiledGraphNode::Clip {
            clip,
            playback_speed,
            looping,
        } = &self.nodes[clip_slot.index()]
        else {
            unreachable!("graph contribution owners are clip nodes")
        };
        let target_mask = context.target_mask.map(|mask_slot| {
            let CompiledGraphNode::Mask { target_mask, .. } = &self.nodes[mask_slot.index()] else {
                unreachable!("graph mask contexts are owned by mask nodes")
            };
            Arc::clone(target_mask)
        });
        CompiledGraphClipInstance {
            clip: clip.clone(),
            playback_speed: *playback_speed,
            looping: *looping,
            weight,
            blend_mode: if context.additive {
                AnimationGraphBlendMode::Additive
            } else {
                AnimationGraphBlendMode::Base
            },
            target_mask,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct GraphEvaluationContext {
    target_mask: Option<GraphNodeSlot>,
    additive: bool,
}

#[derive(Clone, Copy, Debug)]
struct GraphClipContribution {
    clip: GraphNodeSlot,
    context: GraphEvaluationContext,
    weight: Real,
}

#[derive(Debug, Default)]
enum GraphContextWeights {
    #[default]
    Empty,
    One {
        context: GraphEvaluationContext,
        weight: Real,
    },
    Many(BTreeMap<GraphEvaluationContext, Real>),
}

impl GraphContextWeights {
    fn accumulate(&mut self, context: GraphEvaluationContext, weight: Real) {
        match self {
            Self::Empty => *self = Self::One { context, weight },
            Self::One {
                context: current_context,
                weight: current_weight,
            } if *current_context == context => *current_weight += weight,
            Self::One {
                context: current_context,
                weight: current_weight,
            } => {
                let mut weights = BTreeMap::new();
                weights.insert(*current_context, *current_weight);
                weights.insert(context, weight);
                *self = Self::Many(weights);
            }
            Self::Many(weights) => {
                *weights.entry(context).or_default() += weight;
            }
        }
    }

    fn for_each(&self, mut visit: impl FnMut(GraphEvaluationContext, Real)) {
        match self {
            Self::Empty => {}
            Self::One { context, weight } => visit(*context, *weight),
            Self::Many(weights) => {
                for (context, weight) in weights {
                    visit(*context, *weight);
                }
            }
        }
    }

    fn propagate(
        &self,
        target: &mut Self,
        scale: Real,
        target_mask: Option<GraphNodeSlot>,
        force_additive: bool,
    ) {
        self.for_each(|mut context, weight| {
            if let Some(target_mask) = target_mask {
                context.target_mask = Some(target_mask);
            }
            context.additive |= force_additive;
            target.accumulate(context, weight * scale);
        });
    }
}

fn parameter_scalar(
    parameters: &[CompiledParameter],
    overrides: &AnimationParameterMap,
    slot: Option<ParameterSlot>,
) -> Option<Real> {
    match parameter_value(parameters, overrides, slot?)? {
        AnimationParameterValue::Scalar(value) => Some(*value),
        AnimationParameterValue::Integer(value) => Some(*value as Real),
        AnimationParameterValue::Bool(value) => Some(if *value { 1.0 } else { 0.0 }),
        _ => None,
    }
}

fn parameter_value<'a>(
    parameters: &'a [CompiledParameter],
    overrides: &'a AnimationParameterMap,
    slot: ParameterSlot,
) -> Option<&'a AnimationParameterValue> {
    let parameter = parameters.get(slot.index())?;
    Some(
        overrides
            .get(&parameter.name)
            .filter(|value| parameter_is_finite(value))
            .unwrap_or(&parameter.default_value),
    )
}

fn parameter_is_finite(value: &AnimationParameterValue) -> bool {
    match value {
        AnimationParameterValue::Scalar(value) => value.is_finite(),
        AnimationParameterValue::Vec2(value) => value.iter().all(|value| value.is_finite()),
        AnimationParameterValue::Vec3(value) => value.iter().all(|value| value.is_finite()),
        AnimationParameterValue::Vec4(value) => value.iter().all(|value| value.is_finite()),
        AnimationParameterValue::Bool(_)
        | AnimationParameterValue::Integer(_)
        | AnimationParameterValue::Trigger => true,
    }
}
