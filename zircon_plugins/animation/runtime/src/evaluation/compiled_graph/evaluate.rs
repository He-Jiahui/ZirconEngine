use std::sync::Arc;

use zircon_runtime::core::framework::animation::{
    AnimationGraphBlendMode, AnimationParameterMap, AnimationParameterValue,
};
use zircon_runtime::core::math::Real;

use super::types::{
    CompiledAnimationGraph, CompiledAnimationGraphEvaluation, CompiledGraphClipInstance,
    CompiledGraphNode, GraphNodeSlot, ParameterSlot,
};

impl CompiledAnimationGraph {
    pub fn evaluate(&self, overrides: &AnimationParameterMap) -> CompiledAnimationGraphEvaluation {
        let parameters = self
            .parameters
            .iter()
            .map(|parameter| {
                overrides
                    .get(&parameter.name)
                    .filter(|value| parameter_is_finite(value))
                    .cloned()
                    .unwrap_or_else(|| parameter.default_value.clone())
            })
            .collect::<Vec<_>>();
        let mut clips = Vec::new();
        collect_clips(
            self.output,
            &self.nodes,
            &parameters,
            None,
            1.0,
            AnimationGraphBlendMode::Base,
            &mut clips,
        );
        CompiledAnimationGraphEvaluation { clips }
    }
}

fn collect_clips(
    slot: GraphNodeSlot,
    nodes: &[CompiledGraphNode],
    parameters: &[AnimationParameterValue],
    inherited_mask: Option<Arc<[bool]>>,
    inherited_weight: Real,
    inherited_mode: AnimationGraphBlendMode,
    output: &mut Vec<CompiledGraphClipInstance>,
) {
    match &nodes[slot.index()] {
        CompiledGraphNode::Clip {
            clip,
            playback_speed,
            looping,
        } => output.push(CompiledGraphClipInstance {
            clip: clip.clone(),
            playback_speed: *playback_speed,
            looping: *looping,
            weight: inherited_weight,
            blend_mode: inherited_mode,
            target_mask: inherited_mask,
        }),
        CompiledGraphNode::Blend {
            inputs,
            weight_parameter,
        } => {
            let scalar = parameter_scalar(parameters, *weight_parameter)
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            let input_count = inputs.len().max(1);
            for (index, input) in inputs.iter().enumerate() {
                let weight = if input_count == 1 {
                    1.0
                } else if index == 0 {
                    1.0 - scalar
                } else {
                    scalar / (input_count - 1) as Real
                };
                collect_clips(
                    *input,
                    nodes,
                    parameters,
                    inherited_mask.clone(),
                    inherited_weight * weight,
                    inherited_mode,
                    output,
                );
            }
        }
        CompiledGraphNode::Additive {
            base,
            additive,
            weight_parameter,
        } => {
            collect_clips(
                *base,
                nodes,
                parameters,
                inherited_mask.clone(),
                inherited_weight,
                inherited_mode,
                output,
            );
            let weight = parameter_scalar(parameters, *weight_parameter)
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            collect_clips(
                *additive,
                nodes,
                parameters,
                inherited_mask,
                inherited_weight * weight,
                AnimationGraphBlendMode::Additive,
                output,
            );
        }
        CompiledGraphNode::Mask { input, target_mask } => collect_clips(
            *input,
            nodes,
            parameters,
            Some(Arc::clone(target_mask)),
            inherited_weight,
            inherited_mode,
            output,
        ),
    }
}

fn parameter_scalar(
    parameters: &[AnimationParameterValue],
    slot: Option<ParameterSlot>,
) -> Option<Real> {
    match parameters.get(slot?.index())? {
        AnimationParameterValue::Scalar(value) => Some(*value),
        AnimationParameterValue::Integer(value) => Some(*value as Real),
        AnimationParameterValue::Bool(value) => Some(if *value { 1.0 } else { 0.0 }),
        _ => None,
    }
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
