//! 条件程序持有参数布局和指令；调用者以同一布局投影参数后求值。
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

    /// 按编译时收集的参数名取值；缺失值不满足比较，无条件全满足表达式按编译契约求值。
    pub fn evaluate(&self, parameters: &AnimationParameterMap) -> bool {
        let values = self
            .parameter_names
            .iter()
            .map(|name| parameters.get(name))
            .collect::<Vec<_>>();
        self.program.evaluate(values.as_slice())
    }
}
