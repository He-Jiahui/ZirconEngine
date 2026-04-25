use serde::{Deserialize, Serialize};

use super::ActivityWindowId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorMainFrameLayout {
    pub active_window: ActivityWindowId,
    pub window_tabs: Vec<ActivityWindowId>,
}
