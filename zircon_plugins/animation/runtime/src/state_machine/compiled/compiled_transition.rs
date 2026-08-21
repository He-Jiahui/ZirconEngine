use std::sync::Arc;

use crate::TransitionDesc;
use crate::state_machine::condition_expression::CompiledConditionProgram;

use super::StateSlot;

#[derive(Clone, Debug)]
pub(super) struct CompiledTransition {
    pub(super) to: StateSlot,
    pub(super) desc: TransitionDesc,
    pub(super) conditions: CompiledConditionProgram,
    pub(super) consumed_triggers: Arc<[String]>,
}
