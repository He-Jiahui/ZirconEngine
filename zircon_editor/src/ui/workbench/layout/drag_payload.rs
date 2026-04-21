use serde::{Deserialize, Serialize};

use crate::ui::workbench::view::ViewInstanceId;
use crate::ui::workbench::view::ViewKind;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DragPayload {
    pub instance_id: ViewInstanceId,
    pub kind: ViewKind,
}
