use serde::{Deserialize, Serialize};

use crate::core::editor_message::SelectionDomain;
use crate::core::play::WorldDomain;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 选择修订或对象焦点的轻量通知；消费者按世界/选择域回查权威选择状态，不能用实体数字跨域定位。
pub enum FocusMessage {
    SelectionChanged {
        domain: SelectionDomain,
        revision: u64,
    },
    FocusObject {
        domain: WorldDomain,
        entity: u64,
    },
}

#[cfg(test)]
#[path = "tests/focus.rs"]
mod tests;
