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
        let mut clips = Vec::new();
        collect_clips(
            self.output,
            &self.nodes,
            &self.parameters,
            overrides,
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
    parameters: &[CompiledParameter],
    overrides: &AnimationParameterMap,
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
            let scalar = parameter_scalar(parameters, overrides, *weight_parameter)
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
                    overrides,
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
                overrides,
                inherited_mask.clone(),
                inherited_weight,
                inherited_mode,
                output,
            );
            let weight = parameter_scalar(parameters, overrides, *weight_parameter)
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            collect_clips(
                *additive,
                nodes,
                parameters,
                overrides,
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
            overrides,
            Some(Arc::clone(target_mask)),
            inherited_weight,
            inherited_mode,
            output,
        ),
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
