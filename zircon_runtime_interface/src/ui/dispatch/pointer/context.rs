use serde::{Deserialize, Serialize};

use crate::ui::dispatch::UiDispatchPhase;
use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::UiPointerRoute;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerDispatchContext {
    pub node_id: UiNodeId,
    #[serde(default = "default_pointer_dispatch_phase")]
    pub phase: UiDispatchPhase,
    pub route: UiPointerRoute,
}

const fn default_pointer_dispatch_phase() -> UiDispatchPhase {
    UiDispatchPhase::Target
}
