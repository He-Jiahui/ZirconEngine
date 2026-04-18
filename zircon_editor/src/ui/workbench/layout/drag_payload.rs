use serde::{Deserialize, Serialize};

use crate::{ViewInstanceId, ViewKind};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DragPayload {
    pub instance_id: ViewInstanceId,
    pub kind: ViewKind,
}
