use crate::state_machine::condition_expression::CompiledConditionProgram;
use crate::TransitionDesc;

use super::StateSlot;

#[derive(Clone, Debug)]
pub(super) struct CompiledTransition {
    pub(super) to: StateSlot,
    pub(super) desc: TransitionDesc,
    pub(super) conditions: CompiledConditionProgram,
}
