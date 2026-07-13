use zircon_runtime::core::framework::animation::{
    AnimationStateMachineAsset, AnimationStateMachineLayerBlendModeAsset,
};

use crate::{MaskWeights, PoseLayerBlendMode};

use super::{CompiledStateMachineLayer, CompiledStateMachineLayers, StateMachineLayerCompileError};

impl CompiledStateMachineLayers {
    pub fn compile(
        source: &AnimationStateMachineAsset,
    ) -> Result<Self, StateMachineLayerCompileError> {
        let mut layers = Vec::with_capacity(source.layers.len());
        for layer in &source.layers {
            if !layer.weight.is_finite() || !(0.0..=1.0).contains(&layer.weight) {
                return Err(StateMachineLayerCompileError::InvalidWeight {
                    layer: layer.name.clone(),
                    weight: layer.weight,
                });
            }
            let mask = (!layer.mask_weights.is_empty())
                .then(|| MaskWeights::try_from_weights(layer.mask_weights.clone()))
                .transpose()
                .map_err(|_| StateMachineLayerCompileError::InvalidMask {
                    layer: layer.name.clone(),
                })?;
            layers.push(CompiledStateMachineLayer {
                name: layer.name.clone(),
                machine: layer.state_machine.clone(),
                weight: layer.weight,
                blend_mode: match layer.blend_mode {
                    AnimationStateMachineLayerBlendModeAsset::Override => {
                        PoseLayerBlendMode::Override
                    }
                    AnimationStateMachineLayerBlendModeAsset::Additive => {
                        PoseLayerBlendMode::Additive
                    }
                },
                mask,
            });
        }
        let mut base = source.clone();
        base.layers.clear();
        Ok(Self {
            base,
            layers: layers.into_boxed_slice(),
        })
    }
}
