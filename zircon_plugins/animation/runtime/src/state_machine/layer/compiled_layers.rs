use super::CompiledStateMachineLayer;

#[derive(Clone, Debug)]
pub struct CompiledStateMachineLayers {
    pub(super) layers: Box<[CompiledStateMachineLayer]>,
}

impl CompiledStateMachineLayers {
    pub fn layers(&self) -> &[CompiledStateMachineLayer] {
        &self.layers
    }
}
