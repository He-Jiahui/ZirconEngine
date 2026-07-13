use zircon_runtime::core::framework::animation::AnimationParameterMap;

use super::instruction::ConditionInstruction;

#[derive(Clone, Debug)]
pub struct CompiledConditionExpression {
    pub(super) parameter_names: Box<[String]>,
    pub(super) program: CompiledConditionProgram,
}

#[derive(Clone, Debug)]
pub(in crate::state_machine) struct CompiledConditionProgram {
    pub(super) instructions: Box<[ConditionInstruction]>,
}

impl CompiledConditionExpression {
    pub fn parameter_count(&self) -> usize {
        self.parameter_names.len()
    }

    pub fn evaluate(&self, parameters: &AnimationParameterMap) -> bool {
        let values = self
            .parameter_names
            .iter()
            .map(|name| parameters.get(name))
            .collect::<Vec<_>>();
        self.program.evaluate(&values)
    }
}
