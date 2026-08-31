use zircon_runtime::core::framework::animation::compiler::state_machine::{
    compile_animation_state_machine, AnimationCompiledStateMachineLayer,
};
use zircon_runtime::core::framework::animation::{
    AnimationStateMachineAsset, AnimationStateMachineLayerBlendModeAsset,
};

use crate::{MaskWeights, PoseLayerBlendMode};

use super::{CompiledStateMachineLayer, CompiledStateMachineLayers, StateMachineLayerCompileError};

impl CompiledStateMachineLayers {
    pub(crate) fn from_compiled(source: &[AnimationCompiledStateMachineLayer]) -> Self {
        let mut layers = Vec::with_capacity(source.len());
        for layer in source {
            let mask = (!layer.mask_weights().is_empty())
                .then(|| MaskWeights::from_validated_weights(layer.mask_weights()));
            layers.push(CompiledStateMachineLayer {
                name: layer.name().to_string(),
                machine: layer.state_machine().clone(),
                weight: layer.weight(),
                blend_mode: match layer.blend_mode() {
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
        Self {
            layers: layers.into_boxed_slice(),
        }
    }
}

pub fn compile_animation_state_machine_layers_runtime(
    source: &AnimationStateMachineAsset,
) -> Result<CompiledStateMachineLayers, StateMachineLayerCompileError> {
    let compilation = compile_animation_state_machine(source);
    let Some(artifact) = compilation.artifact() else {
        return Err(StateMachineLayerCompileError::SourceDiagnostics(
            compilation.diagnostics().to_vec(),
        ));
    };
    Ok(CompiledStateMachineLayers::from_compiled(artifact.layers()))
}
