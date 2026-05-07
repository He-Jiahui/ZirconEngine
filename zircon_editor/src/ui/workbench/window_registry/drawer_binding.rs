use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::ActivityWindowId;
use crate::ui::workbench::view::ViewInstanceId;

use super::DrawerDockPosition;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawerBinding {
    pub window_id: ActivityWindowId,
    pub drawer_view: ViewInstanceId,
    pub dock_position: DrawerDockPosition,
}

impl DrawerBinding {
    pub fn new(
        window_id: ActivityWindowId,
        drawer_view: ViewInstanceId,
        dock_position: DrawerDockPosition,
    ) -> Self {
        Self {
            window_id,
            drawer_view,
            dock_position,
        }
    }
}
