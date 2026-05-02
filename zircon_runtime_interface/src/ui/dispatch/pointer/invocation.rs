use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::UiPointerDispatchEffect;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiPointerDispatchInvocation {
    pub node_id: UiNodeId,
    pub effect: UiPointerDispatchEffect,
}
