//! 状态机全部编译层的有序集合，供管线一次性组合覆盖和加法结果。
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
