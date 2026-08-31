use std::collections::BTreeMap;
use std::sync::Arc;

use super::{CompiledState, CompiledTransition, StateSlot};

#[derive(Clone, Debug)]
pub struct CompiledAnimationStateMachine {
    pub(super) states: Box<[CompiledState]>,
    pub(super) state_slots: BTreeMap<String, StateSlot>,
    pub(super) parameter_names: Arc<[String]>,
    pub(super) entry: StateSlot,
    pub(super) transitions: Box<[Box<[CompiledTransition]>]>,
}
