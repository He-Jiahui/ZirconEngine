use zircon_runtime::core::framework::animation::AnimationStateMachineAsset;

use super::CompiledStateMachineLayer;

#[derive(Clone, Debug)]
pub struct CompiledStateMachineLayers {
    pub(super) base: AnimationStateMachineAsset,
    pub(super) layers: Box<[CompiledStateMachineLayer]>,
}

impl CompiledStateMachineLayers {
    pub fn base(&self) -> &AnimationStateMachineAsset {
        &self.base
    }

    pub fn layers(&self) -> &[CompiledStateMachineLayer] {
        &self.layers
    }
}
