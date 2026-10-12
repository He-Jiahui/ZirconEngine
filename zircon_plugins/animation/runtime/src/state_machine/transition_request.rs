//! 稠密源状态和目标状态的转换请求；实例推进在同一编译状态机内消费。
use super::{TransitionDesc, TransitionState};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionRequest {
    pub(super) from: TransitionState,
    pub(super) to: TransitionState,
    pub(super) desc: TransitionDesc,
}

impl TransitionRequest {
    pub const fn new(from: TransitionState, to: TransitionState, desc: TransitionDesc) -> Self {
        Self { from, to, desc }
    }
}
