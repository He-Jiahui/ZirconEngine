use serde::{Deserialize, Serialize};

use crate::ui::dispatch::UiDispatchPhase;
use crate::ui::event_ui::UiNodeId;

use super::UiPointerDispatchEffect;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerDispatchInvocation {
    pub node_id: UiNodeId,
    #[serde(default = "default_pointer_dispatch_phase")]
    pub phase: UiDispatchPhase,
    pub effect: UiPointerDispatchEffect,
}

const fn default_pointer_dispatch_phase() -> UiDispatchPhase {
    UiDispatchPhase::Target
}
