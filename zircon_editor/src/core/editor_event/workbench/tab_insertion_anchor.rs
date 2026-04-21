use serde::{Deserialize, Serialize};

use super::{TabInsertionSide, ViewInstanceId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabInsertionAnchor {
    pub target_id: ViewInstanceId,
    pub side: TabInsertionSide,
}
