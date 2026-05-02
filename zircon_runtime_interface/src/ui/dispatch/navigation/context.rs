use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::UiNavigationRoute;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationDispatchContext {
    pub node_id: UiNodeId,
    pub route: UiNavigationRoute,
}
