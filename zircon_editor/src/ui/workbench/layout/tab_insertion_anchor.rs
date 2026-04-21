use serde::{Deserialize, Serialize};

use crate::ui::workbench::view::ViewInstanceId;

use super::TabInsertionSide;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabInsertionAnchor {
    pub target_id: ViewInstanceId,
    pub side: TabInsertionSide,
}
