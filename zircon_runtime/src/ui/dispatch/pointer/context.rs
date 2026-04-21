use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::UiPointerRoute;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerDispatchContext {
    pub node_id: UiNodeId,
    pub route: UiPointerRoute,
}
