use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::UiNavigationDispatchEffect;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationDispatchInvocation {
    pub node_id: UiNodeId,
    pub effect: UiNavigationDispatchEffect,
}
