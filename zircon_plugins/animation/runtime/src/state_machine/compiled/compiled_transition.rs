//! 从一个编译状态到目标状态的转换记录；条件程序和触发器集合由源编译产物生成。
use std::sync::Arc;

use crate::state_machine::condition_expression::CompiledConditionProgram;
use crate::TransitionDesc;

use super::StateSlot;

#[derive(Clone, Debug)]
pub(super) struct CompiledTransition {
    pub(super) to: StateSlot,
    pub(super) desc: TransitionDesc,
    pub(super) conditions: CompiledConditionProgram,
    pub(super) consumed_triggers: Arc<[String]>,
}
