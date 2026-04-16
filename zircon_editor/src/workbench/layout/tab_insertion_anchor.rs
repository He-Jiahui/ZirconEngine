use serde::{Deserialize, Serialize};

use crate::ViewInstanceId;

use super::TabInsertionSide;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabInsertionAnchor {
    pub target_id: ViewInstanceId,
    pub side: TabInsertionSide,
}
