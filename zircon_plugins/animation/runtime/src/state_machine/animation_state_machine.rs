//! 某一资产修订的状态机稠密布局；实例评估通过稳定状态名进入布局，并按声明次序选择转换。
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
